use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;

use crate::amp_envelope::AmpEnvelope;
use crate::midi::get_linear_bent_freq;
use crate::oscillator::{Oscillator, Waveform};
use crate::voices::Voice;

pub struct DualOscVoice {
    freq: f32,
    midi_note: usize,
    osc_a: Oscillator,
    osc_b: Oscillator,
    lfo_amp: f32,
    lfo: Oscillator,
    last_sample: f32,

    pub envelope: AmpEnvelope,
    pub on: Arc<AtomicBool>,
}

impl DualOscVoice {
    pub fn new(envelope: AmpEnvelope, lfo: Oscillator) -> Self {
        DualOscVoice {
            freq: 0.0,
            midi_note: 0,
            osc_a: Oscillator::new(Waveform::Sine),
            osc_b: Oscillator::new(Waveform::Sine),
            lfo_amp: 0.0,
            lfo,
            last_sample: 0.0,
            envelope,
            on: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_waveform_a(&mut self, waveform: Waveform) {
        self.osc_a.set_waveform(waveform);
    }

    pub fn set_phase_offset_a(&mut self, offset: f32) {
        self.osc_a.set_phase(self.osc_a.phase + offset);
    }

    pub fn set_waveform_b(&mut self, waveform: Waveform) {
        self.osc_b.set_waveform(waveform);
    }

    pub fn set_phase_offset_b(&mut self, offset: f32) {
        self.osc_b.set_phase(self.osc_b.phase + offset);
    }

    pub fn set_lfo_amp(&mut self, amp: f32) {
        self.lfo_amp = amp;
    }

    pub fn set_lfo_freq(&mut self, freq: f32) {
        self.lfo.set_freq(freq);
    }

    pub fn set_releasing(&mut self) {
        self.envelope.set_releasing();
    }

    pub fn get_releasing(&self) -> bool {
        self.envelope.get_releasing()
    }

    pub fn get_release_complete(&self) -> bool {
        self.envelope.get_release_complete()
    }
}

impl Clone for DualOscVoice {
    fn clone(&self) -> Self {
        DualOscVoice {
            freq: self.freq,
            midi_note: self.midi_note,
            osc_a: self.osc_a.clone(),
            osc_b: self.osc_b.clone(),
            lfo_amp: self.lfo_amp,
            lfo: self.lfo.clone(),
            last_sample: self.last_sample,
            envelope: self.envelope.clone(),
            on: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Voice for DualOscVoice {
    fn set_freq(&mut self, freq: f32, midi_note: usize) {
        self.osc_a.set_freq(freq);
        self.osc_b.set_freq(freq);
        self.freq = freq;
        self.midi_note = midi_note;
    }

    fn set_pitch_bend(&mut self, midi_bend: u16) {
        let bent_freq = get_linear_bent_freq(self.midi_note, midi_bend, 2);
        self.set_freq(bent_freq, self.midi_note);
    }
}

impl Iterator for DualOscVoice {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let envelope_amp = match self.envelope.next() {
            Some(amp) => amp,
            None => 1.0,
        };

        let lfo_sample = self.lfo.next_sample();

        let sample_a = self.osc_a.next_sample_with_mod(self.lfo_amp * lfo_sample);
        let sample_b = self.osc_b.next_sample_with_mod(self.lfo_amp * lfo_sample);

        self.last_sample = (envelope_amp * (sample_a + sample_b)).clamp(-1.0, 1.0);

        Some(self.last_sample)
    }
}
