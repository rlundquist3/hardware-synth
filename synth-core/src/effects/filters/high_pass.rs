use core::f32::consts::PI;
use micromath::F32Ext;

use crate::{SAMPLE_RATE, effects::filters::biquad::Biquad};

// reference: https://www.w3.org/TR/audio-eq-cookbook/#formulae

fn get_normalized_coefficients(cutoff_freq: f32, q: f32) -> ((f32, f32, f32), (f32, f32, f32)) {
    let omega = 2.0 * PI * cutoff_freq / SAMPLE_RATE as f32;
    let alpha = omega.sin() / (2.0 * q);

    let a_0 = 1.0 + alpha;

    let b = (
        (1.0 + omega.cos()) / 2.0 / a_0,
        -1.0 * (1.0 + omega.cos()) / a_0,
        (1.0 + omega.cos()) / 2.0 / a_0,
    );
    let a = (1.0, -2.0 * omega.cos() / a_0, (1.0 - alpha) / a_0);

    (a, b)
}

pub fn new(cutoff_freq: f32, q: f32) -> Biquad {
    Biquad::new(
        "High Pass Filter",
        cutoff_freq,
        (1000.0, 20000.0),
        q,
        get_normalized_coefficients,
    )
}
