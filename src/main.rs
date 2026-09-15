#![no_std]
#![no_main]

mod allocator;
mod amp_envelope;
mod audio;
mod display;
mod effects;
mod engines;
mod logger;
mod midi;
mod oscillator;
mod panic;
mod parameter;
mod tinyusb;
mod utils;
mod voices;

extern crate alloc;
use alloc::string::ToString;
use core::cell::RefCell;
use daisy_embassy::{
    default_rcc,
    hal::{bind_interrupts, peripherals},
    new_daisy_board,
};
use defmt_serial as _;
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_stm32::interrupt;
use embassy_stm32::{
    i2c::{Config, I2c},
    interrupt::{InterruptExt, Priority},
    usart,
};
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Timer;
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use crate::{
    audio::audio_handler,
    logger::{log_handler, serial_log},
    midi::{initialize_midi_host, usb_host_task},
    tinyusb::{BOARD_TUH_RHPORT, tusb_int_handler},
};
use crate::{
    display::{DISPLAY, DisplayContent, display_handler},
    engines::fm::{ENGINE, FMSynth, voice_state_handler},
    voices::VOICES,
};

pub static SAMPLE_RATE: u32 = 44_100;

// Bind interrupt for USART1, used for serial logging
bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
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
    let engine = ENGINE.init(Mutex::new(RefCell::new(FMSynth::new())));

    interrupt::USART3.set_priority(Priority::P0);
    let audio_executor = AUDIO_EXECUTOR.start(interrupt::USART3);
    audio_executor.spawn(audio_handler(audio_interface, engine).unwrap());
    audio_executor.spawn(voice_state_handler(engine).unwrap());

    serial_log("Audio Initialized");
    // End audio setup

    // Audio test
    low_priority_spawner.spawn(c_major().unwrap());

    // Start MIDI setup
    interrupt::OTG_HS.set_priority(Priority::P1);
    interrupt::UART4.set_priority(Priority::P2);

    initialize_midi_host(board.pins.d29, board.pins.d30);

    let midi_executor = MIDI_EXECUTOR.start(interrupt::UART4);
    midi_executor.spawn(usb_host_task().unwrap());
    // End MIDI setup

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

    low_priority_spawner.spawn(display_handler(display).unwrap());

    serial_log("Display Initialized");
    // End display setup
}

// TESTING STUFF
#[embassy_executor::task]
async fn c_major() {
    let mut receiver = VOICES.receiver().unwrap();
    let sender = VOICES.sender();

    let _value = receiver.get().await;
    let updated = [
        (true, 60),
        (false, 60),
        (false, 60),
        (false, 60),
        (false, 60),
    ];

    sender.send(updated);
    DISPLAY
        .send(DisplayContent {
            text: "C".to_string(),
        })
        .await;
    Timer::after_millis(500).await;

    let updated = [
        (true, 60),
        (true, 64),
        (false, 60),
        (false, 60),
        (false, 60),
    ];

    sender.send(updated);
    DISPLAY
        .send(DisplayContent {
            text: "C-E".to_string(),
        })
        .await;
    Timer::after_millis(500).await;

    let updated = [(true, 60), (true, 64), (true, 67), (false, 60), (false, 60)];

    sender.send(updated);
    DISPLAY
        .send(DisplayContent {
            text: "C-E-G".to_string(),
        })
        .await;
}
