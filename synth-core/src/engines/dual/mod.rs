use alloc::vec;
use alloc::vec::Vec;
use core::f32::consts::PI;
use core::sync::atomic::Ordering;

use crate::effects::gain::Gain;
use crate::engines::Engine;
use crate::engines::dual::voice::DualOscVoice;
use crate::midi::MIDI_NOTE_FREQS;
use crate::oscillator::{Oscillator, Waveform};
use crate::parameter::ParameterChange::{self, Decrement, Increment};
use crate::parameter::UserParameters;
use crate::utils::{LFO_AMP_RANGE, LFO_FREQ_RANGE};
use crate::voices::{Voice, Voices};
use crate::{amp_envelope::AmpEnvelope, effects::Effect, parameter::Parameter};

mod voice;

const VOICE_COUNT: usize = 6;

pub struct DualOsc {
    pub voices: Voices<DualOscVoice>,
    headroom_gain: Gain,
    envelope: AmpEnvelope,
    parameters: Vec<Parameter>,
}

impl DualOsc {
    pub fn new() -> Self {
        let envelope = AmpEnvelope::new(0.3, 0.3, 0.9, 0.2);
        let lfo = Oscillator::new(Waveform::Sine);
        let voice = DualOscVoice::new(envelope.clone(), lfo.clone());
        let voices = Voices::new((0..VOICE_COUNT).map(|_| voice.clone()).collect());

        let mut headroom_gain = Gain::new(-16.0);
        headroom_gain.toggle();

        DualOsc {
            voices,
            headroom_gain,
            envelope,
            parameters: vec![
                Parameter::new("Wave A", 1.0, 1.0, (1.0, 4.0)),
                Parameter::new("Offset A", 0.0, PI / 8.0, (-2.0 * PI, 2.0 * PI)),
                Parameter::new("Wave B", 1.0, 1.0, (1.0, 4.0)),
                Parameter::new("Offset B", PI / 4.0, PI / 8.0, (-2.0 * PI, 2.0 * PI)),
                Parameter::new("LFO Amp", 0.0, 0.025, LFO_AMP_RANGE),
                Parameter::new("LFO Freq", 0.0, 0.25, LFO_FREQ_RANGE),
            ],
        }
    }
}

impl Engine for DualOsc {
    fn note_on(&mut self, note: u8) {
        let voice = self.voices.voice_on(note);
        voice.set_freq(MIDI_NOTE_FREQS[note as usize], note as usize);
        voice.on.store(true, Ordering::Relaxed);
    }

    fn note_off(&mut self, note: u8) {
        if let Some(voice) = self.voices.voice_off(note) {
            voice.set_releasing();
            voice.on.store(false, Ordering::Relaxed);
        }
    }

    fn set_pitch_bend(&mut self, bend: u16) {
        self.voices.voices.iter_mut().for_each(|voice| {
            voice.set_pitch_bend(bend);
        });
    }

    fn get_envelope_parameters(&self) -> &[Parameter] {
        self.envelope.get_parameters()
    }

    fn get_name(&self) -> &'static str {
        "Dual"
    }
}

impl Iterator for DualOsc {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let raw = self.voices.next()?;
        let sample = self.headroom_gain.process(raw).clamp(-1.0, 1.0);

        Some(sample)
    }
}

impl Iterator for Voices<DualOscVoice> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.iter_mut().fold(0.0, |acc: f32, voice| {
            if voice.on.load(Ordering::Relaxed) || !voice.get_release_complete() {
                acc + voice.next().unwrap_or(0.0)
            } else {
                acc
            }
        }))
    }
}

impl UserParameters for DualOsc {
    fn get_parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let synth_param_count = self.parameters.len();

        if index >= synth_param_count {
            self.voices.voices.iter_mut().for_each(|voice| {
                voice
                    .envelope
                    .update_parameter(index - synth_param_count, change.clone());
            });
            return self
                .envelope
                .update_parameter(index - synth_param_count, change);
        }

        let param: &mut Parameter = self.parameters.get_mut(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);

        param.set_value(updated_value);

        match index {
            0 => {
                let waveform = match updated_value {
                    1.0 => Waveform::Sine,
                    2.0 => Waveform::Sawtooth,
                    3.0 => Waveform::Square,
                    4.0 => Waveform::Triangle,
                    _ => Waveform::Sine,
                };
                self.voices.voices.iter_mut().for_each(|voice| {
                    voice.set_waveform_a(waveform.clone());
                });
                Some(param.clone())
            }
            1 => {
                self.voices.voices.iter_mut().for_each(|voice| {
                    voice.set_phase_offset_a(updated_value);
                });
                Some(param.clone())
            }
            2 => {
                let waveform = match updated_value {
                    1.0 => Waveform::Sine,
                    2.0 => Waveform::Sawtooth,
                    3.0 => Waveform::Square,
                    4.0 => Waveform::Triangle,
                    _ => Waveform::Sine,
                };
                self.voices.voices.iter_mut().for_each(|voice| {
                    voice.set_waveform_b(waveform.clone());
                });
                Some(param.clone())
            }
            3 => {
                self.voices.voices.iter_mut().for_each(|voice| {
                    voice.set_phase_offset_b(updated_value);
                });
                Some(param.clone())
            }
            4 => {
                self.voices
                    .voices
                    .iter_mut()
                    .for_each(|voice| voice.set_lfo_amp(updated_value));
                Some(param.clone())
            }
            5 => {
                self.voices
                    .voices
                    .iter_mut()
                    .for_each(|voice| voice.set_lfo_freq(updated_value));
                Some(param.clone())
            }
            _ => None,
        }
    }
}
