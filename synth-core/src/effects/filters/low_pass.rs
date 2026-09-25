use core::f32::consts::PI;
use micromath::F32Ext;

use crate::{SAMPLE_RATE, effects::filters::biquad::Biquad};

fn get_normalized_coefficients(cutoff_freq: f32, q: f32) -> ((f32, f32, f32), (f32, f32, f32)) {
    let k: f32 = (PI * cutoff_freq / SAMPLE_RATE as f32).tan();
    let a_0 = 1.0 + k / q + k.powf(2.0);

    let b = (
        k.powf(2.0) / a_0,
        2.0 * k.powf(2.0) / a_0,
        k.powf(2.0) / a_0,
    );
    let a = (
        1.0,
        (2.0 * k.powf(2.0) - 2.0) / a_0,
        (1.0 - k / q + k.powf(2.0)) / a_0,
    );

    (a, b)
}

pub fn new(cutoff_freq: f32, q: f32) -> Biquad {
    Biquad::new(
        "Low Pass Filter",
        cutoff_freq,
        (0.0, 600.0),
        q,
        get_normalized_coefficients,
    )
}
