use core::f32::consts::PI;
use micromath::F32Ext;

use crate::SAMPLE_RATE;

#[derive(Clone, Debug)]
pub enum Waveform {
    Sine,
}
pub use self::Waveform::*;

/**
 * TODO: implement lookup table for trig functions if micromath isn't fast enough. try it out for now
 */

#[derive(Clone, Debug)]
pub struct Oscillator {
    waveform: Waveform,
    pub freq: f32,
    phase: f32,
    phase_delta: f32,
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

        match self.waveform {
            Sine => self.phase.sin(),
        }
    }
}

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.next_sample())
    }
}
