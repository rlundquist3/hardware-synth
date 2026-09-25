use alloc::vec;
use alloc::{format, vec::Vec};
use heapless::String;

use crate::{
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

pub type NormalizedCoefficientsFn = fn(f32, f32) -> ((f32, f32, f32), (f32, f32, f32));

#[derive(Clone, Debug)]
pub struct Biquad {
    name: String<8>,
    parameters: Vec<Parameter>,
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    s_1: f32,
    s_2: f32,
    get_normalized_coefficients: NormalizedCoefficientsFn,
}

impl Biquad {
    pub fn new(
        name: &str,
        cutoff_freq: f32,
        cutoff_freq_range: (f32, f32),
        q: f32,
        get_normalized_coefficients: NormalizedCoefficientsFn,
    ) -> Self {
        let (a, b) = get_normalized_coefficients(cutoff_freq, q);

        Biquad {
            name: String::try_from(name).unwrap_or(String::new()),
            parameters: vec![
                Parameter::new("Toggle", 0.0, 1.0, (0.0, 1.0), |v| match v {
                    1.0 => format!("on"),
                    _ => format!("off"),
                }),
                Parameter::new("Cutoff", cutoff_freq, 10.0, cutoff_freq_range, |v| {
                    format!("{:.0}Hz", v)
                }),
                Parameter::new("Q", q, 0.1, (0.1, 30.0), |v| format!("{:.1}", v)),
            ],
            a,
            b,
            s_1: 0.0,
            s_2: 0.0,
            get_normalized_coefficients,
        }
    }

    fn recalculate_coefficients(&mut self) {
        let cutoff_freq = self.parameters[1].get_value();
        let q = self.parameters[2].get_value();

        let (a, b) = (self.get_normalized_coefficients)(cutoff_freq, q);

        self.a = a;
        self.b = b;
    }
}

impl Effect for Biquad {
    fn process(&mut self, sample: f32) -> f32 {
        let on = self.parameters[0].get_value();

        if on != 1.0 {
            return sample;
        }

        let result = self.b.0 * sample + self.s_1;
        self.s_1 = self.b.1 * sample + self.s_2 - self.a.1 * result;
        self.s_2 = self.b.2 * sample - self.a.2 * result;

        result
    }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl UserParameters for Biquad {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let mut param = self.parameters.get_mut(index)?.clone();
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        param.set_value((param.get_value() + delta).clamp(param.range.0, param.range.1));

        if index != 0 {
            self.recalculate_coefficients();
        }

        Some(param.clone())
    }
}
