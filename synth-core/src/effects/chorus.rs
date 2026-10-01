use core::ops::Range;

use alloc::vec;
use alloc::{format, vec::Vec};

use rand::random_range;

use crate::{
    SAMPLE_RATE,
    effects::{Effect, EffectComponent, effect_components::lfo_delay_line::LFODelay},
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

const DELAY_RANGE: Range<f32> = 20.0..40.0;
const AMP_RANGE: Range<f32> = 0.05..0.2;
const FREQ_RANGE: Range<f32> = 0.1..4.0;
const MATCH_THRESHOLD: f32 = 0.01;

#[derive(Debug)]
pub struct Chorus {
    on: bool,
    voices: Vec<LFODelay>,
    voice_parameters: Vec<Parameter>,
    voice_parameter_targets: Vec<(f32, f32)>,
    parameters: Vec<Parameter>,
}

impl Chorus {
    pub fn new() -> Self {
        let parameters = vec![Parameter::new("Dry/Wet", 0.3, 0.05, (0.0, 0.5), |v| {
            format!("{:.2}/{:.2}", 1.0 - v, v)
        })];

        let mut voice_parameters = Vec::new();
        let mut voices = Vec::new();
        let mut voice_parameter_targets = Vec::new();

        for _ in 0..4 {
            let d = random_range(DELAY_RANGE);
            let a = random_range(AMP_RANGE);
            let f = random_range(FREQ_RANGE);

            let delay_param =
                Parameter::new("Delay", d, 1.0, (DELAY_RANGE.start, DELAY_RANGE.end), |v| {
                    format!("{:.0}ms", v)
                });
            let amp_param = Parameter::new("Amp", a, 0.05, (AMP_RANGE.start, AMP_RANGE.end), |v| {
                format!("{:.2}", v)
            });
            let freq_param =
                Parameter::new("Freq", f, 0.1, (FREQ_RANGE.start, FREQ_RANGE.end), |v| {
                    format!("{:.1}Hz", v)
                });

            voice_parameters.push(delay_param.clone());
            voice_parameters.push(amp_param.clone());
            voice_parameters.push(freq_param.clone());
            voice_parameter_targets.push((d, 0.0));
            voice_parameter_targets.push((a, 0.0));
            voice_parameter_targets.push((f, 0.0));
            voices.push(LFODelay::new(delay_param, amp_param, freq_param));
        }

        Chorus {
            on: false,
            voices,
            voice_parameters,
            voice_parameter_targets,
            parameters,
        }
    }

    fn adjust_parameters(&mut self) {
        self.voice_parameters
            .iter_mut()
            .enumerate()
            .for_each(|(i, p)| {
                let current = p.get_value();
                let (target, step) = self.voice_parameter_targets[i];

                if (target - current).abs() < MATCH_THRESHOLD {
                    let range = match i % 3 {
                        0 => DELAY_RANGE,
                        1 => AMP_RANGE,
                        2 => FREQ_RANGE,
                        _ => DELAY_RANGE,
                    };
                    let new_target = random_range(range);
                    let new_step =
                        (new_target - current) / (random_range(1.0..5.0) * SAMPLE_RATE as f32);
                    self.voice_parameter_targets[i] = (new_target, new_step);
                } else {
                    p.set_value(current + step);
                }
            });
    }
}

impl Effect for Chorus {
    fn toggle(&mut self) {
        self.on = !self.on;
    }

    fn is_on(&self) -> bool {
        self.on
    }

    fn process(&mut self, sample: f32) -> f32 {
        if !self.on {
            return sample;
        }

        self.adjust_parameters();

        let wet = self.parameters[0].get_value() / self.voices.len() as f32;
        let dry = 1.0 - self.parameters[0].get_value();

        let voice_samples = self
            .voices
            .iter_mut()
            .fold(0.0, |acc, v| acc + wet * v.process(sample));
        let result = dry * sample + voice_samples;

        result
    }

    fn get_name(&self) -> &str {
        "Chorus"
    }
}

impl UserParameters for Chorus {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get_mut(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };

        let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);
        param.set_value(updated_value);

        Some(param.clone())
    }
}
