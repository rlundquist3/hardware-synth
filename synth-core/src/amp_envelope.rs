use alloc::vec::Vec;
use alloc::{format, vec};

use crate::{
    SAMPLE_RATE,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

#[derive(Clone, Debug)]
pub struct AmpEnvelope {
    attack_samples: u32,
    decay_samples: u32,
    sustain: f32,
    release_samples: u32,
    attack_step: f32,
    decay_step: f32,
    release_step: f32,
    elapsed_samples: u32,
    amp: f32,
    releasing: bool,
    release_complete: bool,
    parameters: Vec<Parameter>,
}

impl AmpEnvelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let attack_samples = (attack * SAMPLE_RATE as f32) as u32;
        let decay_samples = (decay * SAMPLE_RATE as f32) as u32;
        let release_samples = (release * SAMPLE_RATE as f32) as u32;

        let attack_step = 1.0 / attack_samples as f32;
        let decay_step = (1.0 - sustain) / decay_samples as f32;
        let release_step = sustain / release_samples as f32;

        AmpEnvelope {
            attack_samples,
            decay_samples,
            sustain,
            release_samples,
            attack_step,
            decay_step,
            release_step,
            elapsed_samples: 0,
            amp: 0.0,
            releasing: false,
            release_complete: true,
            parameters: vec![
                Parameter::new("Attack", attack, 0.1, (0.0, 2.0), |v| format!("{:.1}s", v)),
                Parameter::new("Decay", decay, 0.1, (0.0, 2.0), |v| format!("{:.1}s", v)),
                Parameter::new("Sustain", sustain, 0.1, (0.0, 1.0), |v| format!("{:.1}", v)),
                Parameter::new("Release", release, 0.1, (0.0, 2.0), |v| {
                    format!("{:.1}s", v)
                }),
            ],
        }
    }

    pub fn set_releasing(&mut self) {
        self.elapsed_samples = 0;
        self.releasing = true;
    }

    pub fn get_releasing(&self) -> bool {
        self.releasing
    }

    pub fn get_release_complete(&self) -> bool {
        self.release_complete
    }
}

impl Iterator for AmpEnvelope {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        match self.releasing {
            false => {
                if self.elapsed_samples < self.attack_samples {
                    // attack phase
                    self.release_complete = false;
                    self.amp = self.amp + self.attack_step;
                } else if self.elapsed_samples < self.attack_samples + self.decay_samples {
                    // decay phase
                    self.amp = self.amp - self.decay_step;
                }
                // sustain is no-op
            }
            true => {
                if self.elapsed_samples < self.release_samples {
                    // release phase
                    self.amp = self.amp - self.release_step;
                } else {
                    // release complete - reset
                    self.elapsed_samples = 0;
                    self.releasing = false;
                    self.release_complete = true;
                    self.amp = 0.0;
                    return Some(self.amp);
                }
            }
        }

        self.elapsed_samples += 1;
        Some(self.amp)
    }
}

impl UserParameters for AmpEnvelope {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param: &mut Parameter = self.parameters.get_mut(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        let new_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);
        param.set_value(new_value);

        match index {
            0 => {
                let attack_samples = (new_value * SAMPLE_RATE as f32) as u32;
                self.attack_samples = attack_samples;
                self.attack_step = 1.0 / attack_samples as f32;
            }
            1 => {
                let decay_samples = (new_value * SAMPLE_RATE as f32) as u32;
                self.decay_samples = decay_samples;
                self.decay_step = (1.0 - self.sustain) / decay_samples as f32;
            }
            2 => {
                self.sustain = new_value;
                self.decay_step = (1.0 - new_value) / self.decay_samples as f32;
                self.release_step = new_value / self.release_samples as f32;
            }
            3 => {
                let release_samples = (new_value * SAMPLE_RATE as f32) as u32;
                self.release_samples = release_samples;
                self.release_step = self.sustain / release_samples as f32;
            }
            _ => {}
        }

        Some(param.clone())
    }
}
