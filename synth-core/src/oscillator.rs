use core::f32::consts::PI;
use micromath::F32Ext;

use crate::SAMPLE_RATE;

#[derive(Clone, Copy, Debug)]
pub enum Waveform {
    Sine,
    Sawtooth,
    Square,
}
pub use self::Waveform::*;

/**
 * TODO: implement lookup table for trig functions if micromath isn't fast enough. try it out for now
 */

#[derive(Clone, Debug)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub freq: f32,
    pub phase: f32,
    pub phase_delta: f32,
}

impl Oscillator {
    pub fn new(waveform: Waveform) -> Self {
        Oscillator {
            waveform: waveform,
            freq: 0.0,
            phase: 0.0,
            phase_delta: 0.0,
        }
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.phase_delta = freq * 2.0 * PI / (SAMPLE_RATE as f32);
        self.freq = freq;
    }

    pub fn next_phase(&mut self) -> f32 {
        self.phase += self.phase_delta;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        self.phase
    }

    pub fn next_phase_with_mod(&mut self, modulation: f32) -> f32 {
        let delta = self.freq * (1.0 + modulation) * 2.0 * PI / (SAMPLE_RATE as f32);
        self.phase += delta;

        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        self.phase
    }

    pub fn next_sample(&mut self) -> f32 {
        self.next_phase();

        sample_for_phase(self.waveform, self.phase, self.phase_delta)
    }
}

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.next_sample())
    }
}

/// Logic pulled out of next_sample (and used there) for use with FM and other
/// cases where phase may not just be self.phase
pub fn sample_for_phase(waveform: Waveform, phase: f32, phase_delta: f32) -> f32 {
    match waveform {
        Sine => phase.sin(),
        Sawtooth => phase / PI - 1.0 + poly_blep(waveform, phase, phase_delta),
        Square => {
            (match phase < PI {
                true => 1.0,
                false => -1.0,
            }) + poly_blep(waveform, phase, phase_delta)
        }
    }
}

#[inline]
fn poly_blep_offset(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t_norm = t / dt;
        -t_norm * t_norm + 2.0 * t_norm - 1.0
    } else if t > 1.0 - dt {
        let t_norm = (t - 1.0) / dt;
        t_norm * t_norm + 2.0 * t_norm + 1.0
    } else {
        0.0
    }
}

#[inline]
pub fn poly_blep(waveform: Waveform, phase: f32, phase_delta: f32) -> f32 {
    let t = phase / (2.0 * PI);
    let dt = phase_delta / (2.0 * PI);

    match waveform {
        Sawtooth => -1.0 * poly_blep_offset(t, dt),
        Square => poly_blep_offset(t, dt) - poly_blep_offset((t + 0.5) % 1.0, dt),
        _ => 0.0,
    }
}
