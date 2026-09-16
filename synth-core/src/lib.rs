#![no_std]

mod amp_envelope;
mod effects;
pub mod engines;
pub mod midi;
mod oscillator;
mod parameter;
pub mod utils;
pub mod voices;

extern crate alloc;

pub static SAMPLE_RATE: u32 = 44_100;
