use core::{cell::RefCell, sync::atomic::Ordering};

use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use embassy_time::Timer;
use synth_core::{
    chain::Chain,
    engines::fm::FMSynth,
    midi::{MIDI_NOTE_FREQS, MidiMessage, get_pitch_bend_value},
    voices::Voice,
};

use crate::{SharedChain, midi::MIDI_BUFFER};

#[embassy_executor::task]
pub async fn midi_buffer_handler(chain: &'static SharedChain) {
    let receiver = MIDI_BUFFER.receiver();

    loop {
        let message = receiver.receive().await;

        match message.0 {
            128 => handle_note_off(chain, message).await,
            144 => match message.2 {
                0 => handle_note_off(chain, message).await,
                _ => handle_note_on(chain, message).await,
            },
            224 => handle_pitch_bend(chain, get_pitch_bend_value(message)).await,
            _ => {}
        };
    }
}

/// Pulse middle C to test audio without MIDI controller
#[embassy_executor::task]
pub async fn midi_heartbeat(chain: &'static SharedChain) {
    loop {
        handle_note_on(chain, MidiMessage(144, 60, 100)).await;
        Timer::after_millis(500).await;
        handle_note_off(chain, MidiMessage(128, 60, 0)).await;
        Timer::after_millis(500).await;
    }
}

async fn handle_note_on(chain: &'static SharedChain, message: MidiMessage) {
    let MidiMessage(_status, note, _vel) = message;
    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();
        chain.get_engine().note_on(note);
    });
}

async fn handle_note_off(chain: &'static SharedChain, message: MidiMessage) {
    let MidiMessage(_status, note, _vel) = message;
    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();
        chain.get_engine().note_off(note);
    })
}

async fn handle_pitch_bend(chain: &'static SharedChain, bend: u16) {
    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();
        chain.get_engine().set_pitch_bend(bend);
    })
}
