#![no_std]
#![no_main]

mod allocator;
mod audio;
mod controls;
mod display;
mod midi;
mod panic;

extern crate alloc;
use alloc::{boxed::Box, format};
use core::cell::RefCell;
use daisy_embassy::{
    default_rcc,
    hal::{
        self, bind_interrupts,
        exti::{self, ExtiInput},
        gpio::Pull,
        interrupt, peripherals,
    },
    new_daisy_board,
};
use defmt_serial as _;
use embassy_executor::{InterruptExecutor, Spawner};
// TODO: clean up these imports - should be able to come from daisy_embassy::hal
use embassy_stm32::{
    i2c::{Config, I2c},
    interrupt::{InterruptExt, Priority},
    usart,
};
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use static_cell::StaticCell;

use crate::{
    audio::audio_handler,
    controls::{
        control_handler,
        encoders::{encoder_click_handler, encoder_handler},
    },
    midi::{BOARD_TUH_RHPORT, initialize_midi_host, tasks::midi_heartbeat, usb_host_task},
};
use crate::{
    display::{DISPLAY_BUFFER, DisplayContent, display_handler},
    midi::tasks::midi_buffer_handler,
};
use logger::{log_handler, serial_log};
use rust_tinyusb_host::tusb_int_handler;
use synth_core::{chain::Chain, engines::fm::FMSynth};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
    EXTI9_5 => hal::exti::InterruptHandler<interrupt::typelevel::EXTI9_5>;
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
 * | low |  | display, logger, everything else |
 */
#[embassy_executor::main]
async fn main(low_priority_spawner: Spawner) {
    let peripherals = embassy_stm32::init(default_rcc());
    let board = new_daisy_board!(peripherals);

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
    let chain = CHAIN.init(BlockingMutex::new(RefCell::new(Chain::new(Box::new(
        FMSynth::new(),
    )))));

    interrupt::USART3.set_priority(Priority::P0);
    let audio_executor = AUDIO_EXECUTOR.start(interrupt::USART3);
    audio_executor.spawn(audio_handler(audio_interface, chain).unwrap());
    audio_executor.spawn(midi_buffer_handler(chain).unwrap());

    // Comment this out to stop middle C heartbeat
    audio_executor.spawn(midi_heartbeat(chain).unwrap());

    serial_log("Audio Initialized");
    // End audio setup

    // Start MIDI setup
    interrupt::OTG_HS.set_priority(Priority::P1);
    interrupt::UART4.set_priority(Priority::P2);

    initialize_midi_host(board.pins.d29, board.pins.d30);

    let midi_executor = MIDI_EXECUTOR.start(interrupt::UART4);
    midi_executor.spawn(usb_host_task().unwrap());
    // End MIDI setup

    // Start control setup
    let clk_0 = ExtiInput::new(board.pins.d1, peripherals.EXTI11, Pull::Up, Irqs);
    let dt_0 = ExtiInput::new(board.pins.d2, peripherals.EXTI10, Pull::Up, Irqs);
    let sw_0 = ExtiInput::new(board.pins.d3, peripherals.EXTI9, Pull::Up, Irqs);
    low_priority_spawner.spawn(encoder_handler(1, clk_0, dt_0).unwrap());
    low_priority_spawner.spawn(encoder_click_handler(1, sw_0).unwrap());
    // End control setup

    // Start display setup
    let i2c = I2c::new_blocking(
        peripherals.I2C1,
        board.pins.d11,
        board.pins.d12,
        Config::default(), // TODO: might want to make this slower
    );
    let display_interface = I2CDisplayInterface::new_custom_address(i2c, 0x3D);
    let mut display: Ssd1306<
        I2CInterface<I2c<'_, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
    > = Ssd1306::new(
        display_interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    display.init().unwrap();

    low_priority_spawner.spawn(display_handler(display, chain).unwrap());
    low_priority_spawner.spawn(control_handler(chain).unwrap());
    DISPLAY_BUFFER.send(1).await;
    serial_log("Display Initialized");
    // End display setup
}
