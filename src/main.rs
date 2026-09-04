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

use daisy_embassy::{audio::HALF_DMA_BUFFER_LENGTH, default_rcc, led::UserLed, new_daisy_board};
use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use crate::{
    audio::f32_to_sample, engines::fm::fm_synth_voice::FMSynthVoice, midi::notes::MIDI_NOTE_FREQS,
};

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<8192> = emballoc::Allocator::new();
extern crate alloc;

pub static SAMPLE_RATE: u32 = 44_100;

static FREQ: Signal<CriticalSectionRawMutex, f32> = Signal::new();

#[embassy_executor::task]
async fn chromatic_test(mut led: UserLed<'static>) {
    let mut i = 0;

    loop {
        FREQ.signal(MIDI_NOTE_FREQS[60 + i]);
        i = (i + 1) % 12;

        led.off();
        Timer::after_millis(500).await;
        led.on();
        Timer::after_millis(500).await;
    }
}

fn audio_output(voice: &mut FMSynthVoice, output: &mut [u32]) {
    let mut buf = [0; HALF_DMA_BUFFER_LENGTH];

    if let Some(new_freq) = FREQ.try_take() {
        voice.set_fundamental_freq(new_freq);
    }

    buf.chunks_mut(2).for_each(|chunk| {
        let sample = f32_to_sample(voice.next().unwrap_or(0.0));
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

    let mut voice = FMSynthVoice::new();

    spawner.spawn(chromatic_test(led).unwrap());

    let mut interface = (interface.start_interface().await).unwrap();
    interface
        .start_callback(|_input, output| {
            audio_output(&mut voice, output);
        })
        .await
        .unwrap();
}
