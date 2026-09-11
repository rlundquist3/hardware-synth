#![no_std]
#![no_main]

mod amp_envelope;
mod audio;
mod display;
mod effects;
mod engines;
mod logger;
mod midi;
mod oscillator;
mod parameter;
mod tinyusb;
mod utils;
mod voices;

use core::cell::RefCell;

use alloc::string::ToString;
use daisy_embassy::{
    default_rcc,
    hal::{bind_interrupts, peripherals},
    new_daisy_board,
};
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
use {defmt_serial as _, panic_probe as _};

use crate::{
    audio::audio_handler,
    logger::{LOGGER, LogLevel, LogMessage, log_handler},
};
use crate::{
    display::{DISPLAY, DisplayContent, display_handler},
    engines::fm::{ENGINE, FMSynth, voice_state_handler},
    voices::VOICES,
};

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();
extern crate alloc;

pub static SAMPLE_RATE: u32 = 44_100;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

static AUDIO_EXECUTOR: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
fn USART3() {
    unsafe {
        AUDIO_EXECUTOR.on_interrupt();
    }
}

#[embassy_executor::main]
async fn main(low_priority_spawner: Spawner) {
    let peripherals = embassy_stm32::init(default_rcc());
    let board = new_daisy_board!(peripherals);

    // Start logger setup
    let logger: usart::UartTx<'_, embassy_stm32::mode::Blocking> =
        usart::UartTx::new_blocking(peripherals.USART1, board.pins.d13, usart::Config::default())
            .unwrap();
    low_priority_spawner.spawn(log_handler(logger).unwrap());
    LOGGER
        .send(LogMessage {
            level: LogLevel::Info,
            message: "==========================================",
        })
        .await;
    // End logger setup

    // Start audio setup
    let audio_interface = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;
    let audio_interface = (audio_interface.start_interface().await).unwrap();
    let engine = ENGINE.init(Mutex::new(RefCell::new(FMSynth::new())));

    interrupt::USART3.set_priority(Priority::P0);
    let high_priority_executor = AUDIO_EXECUTOR.start(interrupt::USART3);
    high_priority_executor.spawn(audio_handler(audio_interface, engine).unwrap());
    high_priority_executor.spawn(voice_state_handler(engine).unwrap());
    LOGGER
        .send(LogMessage {
            level: LogLevel::Info,
            message: "Audio Initialized",
        })
        .await;
    // End audio setup

    // Audio test
    low_priority_spawner.spawn(c_major().unwrap());

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

    LOGGER
        .send(LogMessage {
            level: LogLevel::Info,
            message: "Display Initialized",
        })
        .await;
    // End display setup

    // Start MIDI setup

    // End MIDI setup
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

    // for i in 0..10 {
    //     DISPLAY
    //         .send(DisplayContent {
    //             text: format!("Test {}", i),
    //         })
    //         .await;
    // }
}
