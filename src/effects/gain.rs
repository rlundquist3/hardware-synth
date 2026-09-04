use alloc::vec;
use alloc::{format, string::String, vec::Vec};

use crate::{
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
    utils::db_to_linear_gain,
};

#[derive(Debug)]
pub struct Gain {
    gain_db: f32,
    gain_linear: f32,
    parameters: Vec<Parameter>,
}

impl Gain {
    pub fn new(gain_db: f32) -> Self {
        Gain {
            gain_db,
            gain_linear: db_to_linear_gain(gain_db),
            parameters: vec![Parameter::new("Gain", gain_db, 0.5, (-16.0, 12.0), |v| {
                format!("{:.1}dB", v)
            })],
        }
    }
}

impl Effect for Gain {
    // fn clone_box(&self) -> Box<dyn Effect> {
    //     Box::new(Gain {
    //         gain_db: self.gain_db,
    //         gain_linear: self.gain_linear,
    //         parameters: self.parameters.clone(),
    //     })
    // }

    fn process(&mut self, sample: f32) -> f32 {
        let current_db = self.parameters[0].get_value();
        if current_db != self.gain_db {
            self.gain_db = current_db;
            self.gain_linear = db_to_linear_gain(current_db);
        }
        self.gain_linear * sample
    }

    fn get_name(&self) -> String {
        String::from("Gain")
    }
}

impl UserParameters for Gain {
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
