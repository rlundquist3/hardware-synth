use alloc::vec;
use alloc::{format, vec::Vec};

use crate::buffer_pool::BufferPool;
use crate::{
    effects::{Effect, EffectComponent, effect_components::lfo_delay_line::LFODelay},
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

#[derive(Debug)]
pub struct Flanger {
    on: bool,
    lfo_delay: LFODelay,
    parameters: Vec<Parameter>,
}

impl Flanger {
    pub fn new(delay_ms: f32, amp: f32, freq: f32, buffer_pool: &mut BufferPool) -> Self {
        let parameters = vec![
            Parameter::new("Delay", delay_ms, 1.0, (0.0, 15.0), |v| {
                format!("{:.0}ms", v)
            }),
            Parameter::new("Amp", amp, 0.05, (0.0, 5.0), |v| format!("{:.2}", v)),
            Parameter::new("Freq", freq, 0.1, (0.0, 10.0), |v| format!("{:.1}Hz", v)),
        ];

        Flanger {
            on: false,
            lfo_delay: LFODelay::new(
                parameters[0].clone(),
                parameters[1].clone(),
                parameters[2].clone(),
                buffer_pool,
            ),
            parameters,
        }
    }
}

impl Effect for Flanger {
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

        let delayed = self.lfo_delay.process(sample);
        let result = 0.5 * sample + 0.5 * delayed;

        result
    }

    fn get_name(&self) -> &'static str {
        "Flanger"
    }
}

impl UserParameters for Flanger {
    fn get_parameters(&self) -> &[Parameter] {
        &self.parameters
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
