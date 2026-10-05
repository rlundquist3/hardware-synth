use alloc::vec;
use alloc::{format, vec::Vec};
use micromath::F32Ext;

use crate::{
    effects::{
        Effect, EffectComponent,
        effect_components::{
            low_pass_feedback_comb_filter::LBCF,
            universal_comb_filter::{UCF, new_ap},
        },
    },
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

// Adapted from https://ccrma.stanford.edu/~jos/pasp/Freeverb.html

#[derive(Debug)]
pub struct Reverb {
    on: bool,
    lbcf_array: Vec<LBCF>,
    ap_array: Vec<UCF>,
    parameters: Vec<Parameter>,
}

impl Reverb {
    pub fn new() -> Self {
        Reverb {
            on: false,
            lbcf_array: vec![
                LBCF::new(0.84, 0.2, 1557),
                LBCF::new(0.84, 0.2, 1617),
                LBCF::new(0.84, 0.2, 1491),
                LBCF::new(0.84, 0.2, 1422),
                LBCF::new(0.84, 0.2, 1277),
                LBCF::new(0.84, 0.2, 1356),
                LBCF::new(0.84, 0.2, 1188),
                LBCF::new(0.84, 0.2, 1116),
            ],
            ap_array: vec![
                new_ap(0.5, 225),
                new_ap(0.5, 556),
                new_ap(0.5, 441),
                new_ap(0.5, 341),
            ],
            parameters: vec![Parameter::new("Dry/Wet", 0.5, 0.05, (0.0, 1.0), |v| {
                format!("{:.2}/{:.2}", 1.0 - v, v)
            })],
        }
    }
}

impl Effect for Reverb {
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

        let lbcf_sum_result = self
            .lbcf_array
            .iter_mut()
            .fold(0.0, |acc, lbcf| acc + lbcf.process(sample));
        let ap_chain_result = self
            .ap_array
            .iter_mut()
            .fold(lbcf_sum_result, |acc, ap| ap.process(acc));

        let wet = self.parameters[0].get_value();
        let dry = 1.0 - wet;
        dry.sqrt() * sample + 1.5 * wet.sqrt() * ap_chain_result
    }

    fn get_name(&self) -> &str {
        "Reverb"
    }
}

impl UserParameters for Reverb {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get_mut(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        param.set_value((param.get_value() + delta).clamp(param.range.0, param.range.1));

        Some(param.clone())
    }
}
