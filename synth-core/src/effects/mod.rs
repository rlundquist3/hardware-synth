pub mod filters;
pub mod gain;

use crate::parameter::UserParameters;

pub trait Effect: UserParameters + Send {
    fn process(&mut self, sample: f32) -> f32;
    fn get_name(&self) -> &str;
}

pub trait EffectComponent {
    fn process(&mut self, sample: f32) -> f32;
}
