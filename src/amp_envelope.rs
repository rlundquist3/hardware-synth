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
    sample_wise_attack_decay_amps: Vec<f32>,
    sample_wise_release_amps: Vec<f32>,
    elapsed_samples: usize,
    amp: f32,
    should_release: bool,
    release_complete: bool,
    parameters: Vec<Parameter>,
}

impl AmpEnvelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        AmpEnvelope {
            attack_samples: 0,
            decay_samples: 0,
            sustain: 0.0,
            release_samples: 0,
            sample_wise_attack_decay_amps: Vec::new(),
            sample_wise_release_amps: Vec::new(),
            elapsed_samples: 0,
            amp: 0.0,
            should_release: false,
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

    fn set_sample_wise_attack_decay_amps(
        &mut self,
        attack_samples: u32,
        decay_samples: u32,
        sustain: f32,
    ) {
        self.attack_samples = attack_samples;
        self.decay_samples = decay_samples;
        self.sustain = sustain;

        self.sample_wise_attack_decay_amps = (0..attack_samples)
            .map(|i| i as f32 / attack_samples as f32)
            .chain(
                (0..decay_samples)
                    .map(|i| 1.0 - (i as f32 / decay_samples as f32) * (1.0 - sustain)),
            )
            .collect();
    }

    fn set_sample_wise_release_amps(&mut self, release_samples: u32, sustain: f32) {
        self.release_samples = release_samples;
        self.sustain = sustain;

        self.sample_wise_release_amps = (0..release_samples)
            .map(|i| sustain - i as f32 / release_samples as f32 * sustain)
            .collect();
    }

    pub fn set_should_release(&mut self) {
        self.elapsed_samples = 0;
        self.should_release = true;
    }

    pub fn get_releasing(&self) -> bool {
        self.should_release
    }

    pub fn get_release_complete(&self) -> bool {
        self.release_complete
    }
}

impl Iterator for AmpEnvelope {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let attack_samples = (self.parameters[0].get_value() * SAMPLE_RATE as f32) as u32;
        let decay_samples = (self.parameters[1].get_value() * SAMPLE_RATE as f32) as u32;
        let sustain = self.parameters[2].get_value();
        let release_samples = (self.parameters[3].get_value() * SAMPLE_RATE as f32) as u32;

        if attack_samples != self.attack_samples
            || decay_samples != self.decay_samples
            || sustain != self.sustain
        {
            self.set_sample_wise_attack_decay_amps(attack_samples, decay_samples, sustain);
        }
        if release_samples != self.release_samples || sustain != self.sustain {
            self.set_sample_wise_release_amps(release_samples, sustain);
        }

        if self.should_release {
            if self.elapsed_samples < self.sample_wise_release_amps.len() {
                self.amp = self.sample_wise_release_amps[self.elapsed_samples];
            } else {
                self.elapsed_samples = 0;
                self.should_release = false;
                self.release_complete = true;
                return Some(0.0);
            }
        } else if self.elapsed_samples < self.sample_wise_attack_decay_amps.len() {
            self.release_complete = false;
            self.amp = self.sample_wise_attack_decay_amps[self.elapsed_samples];
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
        param.set_value((param.get_value() + delta).clamp(param.range.0, param.range.1));

        Some(param.clone())
    }
}
