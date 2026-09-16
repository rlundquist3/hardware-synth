pub mod gain;

use alloc::string::String;

use crate::parameter::UserParameters;

pub trait Effect: UserParameters {
    fn process(&mut self, sample: f32) -> f32;
    // fn clone_box(&self) -> Box<dyn Effect>;
    fn get_name(&self) -> String;
}

pub trait EffectComponent {
    fn process(&mut self, sample: f32) -> f32;
}
