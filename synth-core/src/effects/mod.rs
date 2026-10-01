pub mod chorus;
pub mod echo;
mod effect_components;
pub mod filters;
pub mod flanger;
pub mod gain;
pub mod reverb;
pub mod soft_clipper;
pub mod vibrato;

use crate::parameter::UserParameters;

pub const EFFECT_COUNT: usize = 7;

pub trait Effect: UserParameters + Send {
    fn toggle(&mut self);
    fn is_on(&self) -> bool;
    fn process(&mut self, sample: f32) -> f32;
    fn get_name(&self) -> &str;
}

pub trait EffectComponent {
    fn process(&mut self, sample: f32) -> f32;
}
