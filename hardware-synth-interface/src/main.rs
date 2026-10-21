#![no_std]
#![no_main]

mod allocator;
mod audio;
mod controls;
mod display;
mod i2c;
mod midi;
mod panic;

extern crate alloc;
use core::{cell::RefCell, slice::from_raw_parts_mut};
use daisy_embassy::{
    default_rcc,
    hal::{
        bind_interrupts,
        exti::{self, ExtiInput},
        gpio::Pull,
        i2c::{Config, I2c},
        interrupt,
        interrupt::{InterruptExt, Priority},
        peripherals, usart,
    },
    new_daisy_board,
    sdram::SDRAM_SIZE,
};
use defmt_serial as _;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use embassy_time::{Delay, Timer};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use static_cell::StaticCell;

use crate::{
    audio::audio_handler,
    controls::{
        control_handler,
        dpad::dpad_handler,
        encoders::{encoder_click_handler, encoder_handler},
    },
    display::DISPLAY_ADDRESS,
    i2c::I2C_BUS,
    midi::{BOARD_TUH_RHPORT, initialize_midi_host, tasks::midi_heartbeat, usb_host_task},
};
use crate::{
    display::{DISPLAY_BUFFER, Display, display_handler},
    midi::tasks::midi_buffer_handler,
};
use logger::{log_handler, serial_log};
use rust_tinyusb_host::tusb_int_handler;
use synth_core::{buffer_pool::BufferPool, chain::Chain};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    EXTI0 => exti::InterruptHandler<interrupt::typelevel::EXTI0>;
    EXTI1 => exti::InterruptHandler<interrupt::typelevel::EXTI1>;
    EXTI2 => exti::InterruptHandler<interrupt::typelevel::EXTI2>;
    EXTI3 => exti::InterruptHandler<interrupt::typelevel::EXTI3>;
    EXTI4 => exti::InterruptHandler<interrupt::typelevel::EXTI4>;
    EXTI9_5 => exti::InterruptHandler<interrupt::typelevel::EXTI9_5>;
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

static AUDIO_EXECUTOR: InterruptExecutor = InterruptExecutor::new();
static MIDI_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

// Interrupt for audio executor
#[interrupt]
fn USART3() {
    unsafe {
        AUDIO_EXECUTOR.on_interrupt();
    }
}

// Interrupt for USB executor
#[interrupt]
fn UART4() {
    unsafe {
        MIDI_EXECUTOR.on_interrupt();
    }
}

// Interrupt for OTG_HS, only for tusb_int_handler, which queues events
#[interrupt]
fn OTG_HS() {
    unsafe {
        tusb_int_handler(BOARD_TUH_RHPORT, true);
    }
}

pub type SharedChain = BlockingMutex<CriticalSectionRawMutex, RefCell<Chain>>;

pub static CHAIN: StaticCell<BlockingMutex<CriticalSectionRawMutex, RefCell<Chain>>> =
    StaticCell::new();

/**
 * Task Priorities
 *
 * | priority | vector | logic |
 * | -------- | ------ | ----- |
 * | P0 | `USART3` | audio |
 * | P1 | `OTG_HS` | `tusb_int_handler` - queues events |
 * | P2 | `UART4` | `usb_host_task`, TinyUSB callbacks (MIDI events) |
 * | low |  | non-MIDI controls, display, logger, everything else |
 */
#[embassy_executor::main]
async fn main(low_priority_spawner: Spawner) {
    let peripherals = embassy_stm32::init(default_rcc());
    let board = new_daisy_board!(peripherals);

    let mut core_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut sdram = board
        .sdram
        .build(&mut core_peripherals.MPU, &mut core_peripherals.SCB);
    let backing: &'static mut [f32] = unsafe {
        let pointer = sdram.init(&mut Delay) as *mut f32;
        from_raw_parts_mut(pointer, SDRAM_SIZE / core::mem::size_of::<f32>())
    };
    let mut buffer_pool = BufferPool::from_slice(backing);

    // Start logger setup
    let logger: usart::UartTx<'_, embassy_stm32::mode::Blocking> =
        usart::UartTx::new_blocking(peripherals.USART1, board.pins.d13, usart::Config::default())
            .unwrap();
    low_priority_spawner.spawn(log_handler(logger).unwrap());

    serial_log("================================");
    // End logger setup

    // Start audio setup
    let audio_interface = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;
    let audio_interface = (audio_interface.start_interface().await).unwrap();
    let chain = CHAIN.init(BlockingMutex::new(RefCell::new(Chain::new(
        &mut buffer_pool,
    ))));

    interrupt::USART3.set_priority(Priority::P0);
    let audio_executor = AUDIO_EXECUTOR.start(interrupt::USART3);
    audio_executor.spawn(audio_handler(audio_interface, chain).unwrap());
    audio_executor.spawn(midi_buffer_handler(chain).unwrap());

    // Comment this out to stop middle C heartbeat
    // audio_executor.spawn(midi_heartbeat(chain).unwrap());

    serial_log("Audio Initialized");
    // End audio setup

    // Start MIDI setup
    interrupt::OTG_HS.set_priority(Priority::P1);
    interrupt::UART4.set_priority(Priority::P2);

    initialize_midi_host(board.pins.d29, board.pins.d30);

    let midi_executor = MIDI_EXECUTOR.start(interrupt::UART4);
    midi_executor.spawn(usb_host_task().unwrap());
    // End MIDI setup

    // Start I2C setup (used for D-pad and display)
    let i2c = I2c::new_blocking(
        peripherals.I2C1,
        board.pins.d11,
        board.pins.d12,
        Config::default(),
    );
    let i2c_bus = I2C_BUS.init(BlockingMutex::new(RefCell::new(i2c)));
    let dpad_i2c = I2cDevice::new(i2c_bus);
    let display_i2c = I2cDevice::new(i2c_bus);
    // End I2C setup

    // Start control setup
    let sw_1 = ExtiInput::new(board.pins.d1, peripherals.EXTI11, Pull::Up, Irqs);
    let clk_1 = ExtiInput::new(board.pins.d2, peripherals.EXTI10, Pull::Up, Irqs);
    let dt_1 = ExtiInput::new(board.pins.d3, peripherals.EXTI9, Pull::Up, Irqs);
    let sw_2 = ExtiInput::new(board.pins.d4, peripherals.EXTI8, Pull::Up, Irqs);
    let clk_2 = ExtiInput::new(board.pins.d5, peripherals.EXTI2, Pull::Up, Irqs);
    let dt_2 = ExtiInput::new(board.pins.d6, peripherals.EXTI12, Pull::Up, Irqs);
    let sw_3 = ExtiInput::new(board.pins.d21, peripherals.EXTI4, Pull::Up, Irqs);
    let clk_3 = ExtiInput::new(board.pins.d19, peripherals.EXTI6, Pull::Up, Irqs);
    let dt_3 = ExtiInput::new(board.pins.d18, peripherals.EXTI7, Pull::Up, Irqs);
    let sw_4 = ExtiInput::new(board.pins.d17, peripherals.EXTI1, Pull::Up, Irqs);
    let clk_4 = ExtiInput::new(board.pins.d16, peripherals.EXTI3, Pull::Up, Irqs);
    let dt_4 = ExtiInput::new(board.pins.d15, peripherals.EXTI0, Pull::Up, Irqs);

    let dpad_interrupt = ExtiInput::new(board.pins.d10, peripherals.EXTI5, Pull::Up, Irqs);
    low_priority_spawner.spawn(dpad_handler(dpad_interrupt, dpad_i2c).unwrap());

    low_priority_spawner.spawn(encoder_handler(1, clk_1, dt_1).unwrap());
    low_priority_spawner.spawn(encoder_click_handler(1, sw_1).unwrap());
    low_priority_spawner.spawn(encoder_handler(2, clk_2, dt_2).unwrap());
    low_priority_spawner.spawn(encoder_click_handler(2, sw_2).unwrap());
    low_priority_spawner.spawn(encoder_handler(3, clk_3, dt_3).unwrap());
    low_priority_spawner.spawn(encoder_click_handler(3, sw_3).unwrap());
    low_priority_spawner.spawn(encoder_handler(4, clk_4, dt_4).unwrap());
    low_priority_spawner.spawn(encoder_click_handler(4, sw_4).unwrap());
    low_priority_spawner.spawn(control_handler(chain).unwrap());
    // End control setup

    // Start display setup
    let display_interface = I2CDisplayInterface::new_custom_address(display_i2c, DISPLAY_ADDRESS);
    let mut display: Display = Ssd1306::new(
        display_interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    // Retry rather than panic: this panel can NACK a first write and accept the
    // next one, and a dropped init is not worth taking the synth down for.
    for _ in 0..3 {
        if display.init().is_ok() {
            break;
        }

        Timer::after_millis(50).await;
    }

    low_priority_spawner.spawn(display_handler(display, chain).unwrap());
    DISPLAY_BUFFER.send(1).await;
    serial_log("Display Initialized");
    // End display setup
}
