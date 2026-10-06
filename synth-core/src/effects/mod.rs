pub mod chorus;
pub mod echo;
mod effect_components;
pub mod filters;
pub mod flanger;
pub mod gain;
pub mod reverb;
pub mod soft_clipper;
pub mod vibrato;

use alloc::boxed::Box;
use heapless::Vec;

use crate::parameter::{Parameter, UserParameters};

pub const EFFECT_COUNT: usize = 7;
pub const MAX_EFFECT_PARAMS: usize = 4;

pub trait Effect: UserParameters + Send {
    fn toggle(&mut self);
    fn is_on(&self) -> bool;
    fn process(&mut self, sample: f32) -> f32;
    fn get_name(&self) -> &'static str;
}

pub trait EffectComponent {
    fn process(&mut self, sample: f32) -> f32;
}

pub struct EffectSnapshot {
    pub name: &'static str,
    pub on: bool,
    pub parameters: Vec<Parameter, MAX_EFFECT_PARAMS>,
}

impl EffectSnapshot {
    pub fn from_effect(effect: &dyn Effect) -> Self {
        EffectSnapshot {
            name: effect.get_name(),
            on: effect.is_on(),
            parameters: effect.get_parameters().iter().cloned().collect(),
        }
    }
}
