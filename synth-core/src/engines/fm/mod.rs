use alloc::vec::Vec;
use alloc::{format, vec};
use core::f32::consts::PI;
use core::sync::atomic::Ordering;

use crate::effects::gain::Gain;
use crate::parameter::ParameterChange::{self, Decrement, Increment};
use crate::parameter::UserParameters;
use crate::voices::Voices;
use crate::{
    amp_envelope::AmpEnvelope, effects::Effect, engines::fm::fm_synth_voice::FMSynthVoice,
    parameter::Parameter,
};

pub mod fm_synth_voice;

const VOICE_COUNT: usize = 5;

#[derive(Clone, Copy, Debug)]
pub struct FreqRatio(pub f32, pub f32);

pub const MOD_INDEX_OPTIONS: &[f32] = &[1.0, 2.0, PI, 4.0, 5.0, 2.0 * PI];
pub const MOD_INDEX_RENDER: &[&str] = &["1", "2", "pi", "4", "5", "2 pi"];

#[derive(Debug)]
pub struct FMSynth {
    pub voices: Voices<FMSynthVoice>,
    headroom_gain: Gain,
    envelope: AmpEnvelope,
    parameters: Vec<Parameter>,
    // pub effects: Vec<Box<dyn Effect>>,
}

impl FMSynth {
    pub fn new() -> Self {
        let envelope = AmpEnvelope::new(0.3, 0.2, 0.8, 0.3);
        let signal_source = FMSynthVoice::new(envelope.clone());
        let voices = Voices::new((0..VOICE_COUNT).map(|_| signal_source.clone()).collect());

        // let mut effects: Vec<Box<dyn Effect>> = Vec::new();

        FMSynth {
            voices,
            parameters: vec![
                Parameter::new("C", 1.0, 1.0, (1.0, 10.0), |v| format!("{:.0}", v)),
                Parameter::new("M", 1.0, 1.0, (1.0, 10.0), |v| format!("{:.0}", v)),
                Parameter::new(
                    "Mod Idx",
                    2.0,
                    1.0,
                    (0.0, (MOD_INDEX_OPTIONS.len() - 1) as f32),
                    |v| format!("{}", MOD_INDEX_RENDER[v as usize]),
                ),
                Parameter::new("LFO Amp", 0.0, 0.025, (0.0, 5.0), |v| format!("{:.3}", v)),
                Parameter::new("LFO Freq", 0.0, 0.25, (0.0, 8.0), |v| format!("{:.2}Hz", v)),
            ],
            headroom_gain: Gain::new(-16.0),
            envelope,
            // effects,
        }
    }

    pub fn set_pitch_bend(&mut self, bend: u16) {
        self.voices.voices.iter_mut().for_each(|voice| {
            voice.set_pitch_bend(bend);
        });
    }

    pub fn get_envelope_parameters(&self) -> Vec<Parameter> {
        self.envelope.get_parameters()
    }
}

impl Iterator for FMSynth {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let raw = self.voices.next()?;

        let headroom_corrected = self.headroom_gain.process(raw);
        let sample =
            headroom_corrected /* self
                .effects
                .iter_mut()
                .fold(headroom_corrected, |sample, effect| effect.process(sample))*/
                .clamp(-1.0, 1.0);

        Some(sample)
    }
}

impl Iterator for Voices<FMSynthVoice> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.iter_mut().fold(0.0, |acc: f32, voice| {
            if voice.on.load(Ordering::Relaxed) {
                acc + voice.next().unwrap_or(0.0)
            } else if !voice.get_release_complete() {
                if !voice.get_releasing() {
                    voice.set_releasing();
                }

                acc + voice.next().unwrap_or(0.0)
            } else {
                acc
            }
        }))
    }
}

impl UserParameters for FMSynth {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()

        // previously chained on envelope params, but probably not needed with new GUI
        // self.parameters
        //     .iter()
        //     .cloned()
        //     .chain(self.envelope.get_parameters())
        //     .collect()
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
                self.voices.voices.iter_mut().for_each(|voice| {
                    let existing = voice.get_freq_ratio();
                    voice.set_freq_ratio(FreqRatio(updated_value, existing.1))
                });
                Some(param.clone())
            }
            1 => {
                self.voices.voices.iter_mut().for_each(|voice| {
                    let existing = voice.get_freq_ratio();
                    voice.set_freq_ratio(FreqRatio(existing.0, updated_value))
                });
                Some(param.clone())
            }
            2 => {
                let mod_index = MOD_INDEX_OPTIONS[updated_value as usize];
                self.voices
                    .voices
                    .iter_mut()
                    .for_each(|voice| voice.set_mod_index(mod_index));
                Some(param.clone())
            }
            3 => {
                self.voices
                    .voices
                    .iter_mut()
                    .for_each(|voice| voice.set_lfo_amp(updated_value));
                Some(param.clone())
            }
            4 => {
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
