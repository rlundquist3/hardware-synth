#![no_std]
#![no_main]

mod amp_envelope;
mod audio;
mod effects;
mod engines;
mod midi;
mod oscillator;
mod parameter;
mod utils;
mod voices;

use core::cell::RefCell;

use daisy_embassy::{audio::HALF_DMA_BUFFER_LENGTH, default_rcc, led::UserLed, new_daisy_board};
use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::Mutex;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::{
    audio::f32_to_sample,
    engines::fm::{ENGINE, FMSynth, voice_state_handler},
    voices::VOICES,
};

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();
extern crate alloc;

pub static SAMPLE_RATE: u32 = 44_100;

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
    Timer::after_millis(500).await;

    let updated = [
        (true, 60),
        (true, 64),
        (false, 60),
        (false, 60),
        (false, 60),
    ];

    sender.send(updated);
    Timer::after_millis(500).await;

    let updated = [(true, 60), (true, 64), (true, 67), (false, 60), (false, 60)];

    sender.send(updated);
}

fn audio_output(engine: &mut FMSynth, output: &mut [u32]) {
    let mut buf = [0; HALF_DMA_BUFFER_LENGTH];

    buf.chunks_mut(2).for_each(|chunk| {
        let sample = f32_to_sample(engine.next().unwrap_or(0.0));
        chunk[0] = sample;
        chunk[1] = sample;
    });

    output.copy_from_slice(&buf);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Entrypoint");

    let peripherals = embassy_stm32::init(default_rcc());
    let board = new_daisy_board!(peripherals);

    let interface = board
        .audio_peripherals
        .prepare_interface(Default::default())
        .await;

    let mut led = board.user_led;
    led.on();

    let engine = ENGINE.init(Mutex::new(RefCell::new(FMSynth::new())));
    spawner.spawn(voice_state_handler(engine).unwrap());

    spawner.spawn(c_major().unwrap());

    let mut interface = (interface.start_interface().await).unwrap();
    interface
        .start_callback(|_input, output| {
            engine.lock(|e| {
                audio_output(&mut e.borrow_mut(), output);
            });
        })
        .await
        .unwrap();
}
