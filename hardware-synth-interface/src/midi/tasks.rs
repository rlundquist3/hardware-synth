use core::{cell::RefCell, sync::atomic::Ordering};

use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use embassy_time::Timer;
use synth_core::{
    engines::fm::FMSynth,
    midi::{MIDI_NOTE_FREQS, MidiMessage, get_pitch_bend_value},
    voices::Voice,
};

use crate::midi::MIDI_BUFFER;

#[embassy_executor::task]
pub async fn midi_buffer_handler(
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
) {
    let receiver = MIDI_BUFFER.receiver();

    loop {
        let message = receiver.receive().await;

        match message.0 {
            128 => handle_note_off(engine, message).await,
            144 => match message.2 {
                0 => handle_note_off(engine, message).await,
                _ => handle_note_on(engine, message).await,
            },
            224 => handle_pitch_bend(engine, get_pitch_bend_value(message)).await,
            _ => {}
        };
    }
}

/// Pulse middle C to test audio without MIDI controller
#[embassy_executor::task]
pub async fn midi_heartbeat(
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
) {
    loop {
        handle_note_on(engine, MidiMessage(144, 60, 100)).await;
        Timer::after_millis(500).await;
        handle_note_off(engine, MidiMessage(128, 60, 0)).await;
        Timer::after_millis(500).await;
    }
}

async fn handle_note_on(
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
    message: MidiMessage,
) {
    let MidiMessage(_status, note, _vel) = message;
    engine.lock(|e| {
        let mut engine = e.borrow_mut();

        let voice = engine.voices.voice_on(note);
        voice.set_freq(MIDI_NOTE_FREQS[note as usize], note as usize);
        voice.on.store(true, Ordering::Relaxed);
    });
}

async fn handle_note_off(
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
    message: MidiMessage,
) {
    let MidiMessage(_status, note, _vel) = message;
    engine.lock(|e| {
        let mut engine = e.borrow_mut();
        if let Some(voice) = engine.voices.voice_off(note) {
            voice.on.store(false, Ordering::Relaxed);
        }
    })
}

async fn handle_pitch_bend(
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
    bend: u16,
) {
    engine.lock(|e| {
        let mut engine = e.borrow_mut();
        engine.set_pitch_bend(bend);
    })
}
