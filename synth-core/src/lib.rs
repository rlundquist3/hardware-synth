#![no_std]

pub mod amp_envelope;
pub mod effects;
pub mod engines;
pub mod midi;
pub mod oscillator;
pub mod parameter;
pub mod utils;
pub mod voices;

extern crate alloc;

pub static SAMPLE_RATE: u32 = 44_100;
