use crate::{
    amp_envelope::AmpEnvelope,
    engines::fm::FreqRatio,
    midi::util::{MIDI_NOTE_FREQS, get_linear_bent_freq},
    oscillator::{Oscillator, Waveform::Sine},
    voices::Voice,
};
use alloc::sync::Arc;
use core::{f32::consts::PI, sync::atomic::AtomicBool};
use micromath::F32Ext;

#[derive(Debug)]
pub struct FMSynthVoice {
    fund_freq: f32,
    midi_note: usize,
    freq_ratio: FreqRatio,
    mod_index: f32,
    carrier_amp: f32,
    carrier_osc: Oscillator,
    mod_osc: Oscillator,
    lfo_amp: f32,
    lfo: Oscillator,
    envelope: AmpEnvelope,

    pub on: Arc<AtomicBool>,
}

impl FMSynthVoice {
    pub fn new(envelope: AmpEnvelope) -> Self {
        let mut lfo = Oscillator::new(Sine);
        lfo.set_freq(0.0);

        FMSynthVoice {
            fund_freq: 0.0,
            midi_note: 0,
            freq_ratio: FreqRatio(1.0, 1.0),
            mod_index: PI,
            carrier_amp: 1.0,
            carrier_osc: Oscillator::new(Sine),
            mod_osc: Oscillator::new(Sine),
            lfo_amp: 0.0,
            lfo,
            envelope,
            on: Arc::new(AtomicBool::new(false)),
        }
    }

    /// set frequency of carrier and modulation oscillators based on
    /// fundamental frequency and frequency ratio
    /// for modulation index I = C/M:
    ///     carrier freq = C * fund freq
    ///     mod freq = M * fund freq
    pub fn set_osc_freqs(&mut self, freq: f32) {
        self.carrier_osc.set_freq(self.freq_ratio.0 as f32 * freq);
        self.mod_osc.set_freq(self.freq_ratio.1 as f32 * freq);
    }

    pub fn get_freq_ratio(&self) -> FreqRatio {
        self.freq_ratio
    }

    pub fn set_freq_ratio(&mut self, freq_ratio: FreqRatio) {
        self.freq_ratio = freq_ratio;
    }

    pub fn set_mod_index(&mut self, mod_index: f32) {
        self.mod_index = mod_index;
    }

    pub fn set_lfo_amp(&mut self, amp: f32) {
        self.lfo_amp = amp;
    }

    pub fn set_lfo_freq(&mut self, freq: f32) {
        self.lfo.set_freq(freq);
    }

    pub fn set_pitch_bend(&mut self, midi_bend: u16) {
        let bent_freq = get_linear_bent_freq(self.midi_note, midi_bend, 2);

        self.set_osc_freqs(bent_freq);
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

    // fn get_mod_amp(self) -> f32 {
    //     self.mod_index * self.mod_osc.freq
    // }
}

impl Clone for FMSynthVoice {
    fn clone(&self) -> Self {
        FMSynthVoice {
            fund_freq: self.fund_freq,
            midi_note: self.midi_note,
            freq_ratio: self.freq_ratio,
            mod_index: self.mod_index,
            carrier_amp: self.carrier_amp,
            carrier_osc: self.carrier_osc.clone(),
            mod_osc: self.mod_osc.clone(),
            lfo_amp: self.lfo_amp,
            lfo: self.lfo.clone(),
            envelope: self.envelope.clone(),
            on: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Voice for FMSynthVoice {
    fn set_freq(&mut self, freq: f32, midi_note: usize) {
        self.fund_freq = freq;
        self.midi_note = midi_note;

        self.set_osc_freqs(freq);
    }
}

impl Iterator for FMSynthVoice {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let lfo_sample = self.lfo.next_sample();

        let c = self
            .carrier_osc
            .next_phase_with_mod(self.lfo_amp * lfo_sample);
        let m = self.mod_osc.next_phase();

        let envelope_amp = match self.envelope.next() {
            Some(amp) => amp,
            None => 1.0,
        };

        Some(envelope_amp * self.carrier_amp * (c + self.mod_index * m.sin()).sin())
    }
}
