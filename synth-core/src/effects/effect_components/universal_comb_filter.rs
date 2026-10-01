use crate::effects::EffectComponent;
use alloc::vec;
use alloc::{format, vec::Vec};

const MAX_DELAY_SECS: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct UCF {
    buffer: Vec<f32>,
    write_index: usize,
    gain: f32,
    ff: f32,
    fb: f32,
}

impl UCF {
    pub fn new(gain: f32, ff: f32, fb: f32, delay_samples: u32) -> Self {
        UCF {
            buffer: vec![0.0; delay_samples as usize],
            write_index: 0,
            gain,
            ff,
            fb,
        }
    }
}

impl EffectComponent for UCF {
    fn process(&mut self, sample: f32) -> f32 {
        let max = self.buffer.len();
        let read_index = (self.write_index + 1) % max;
        let delayed = self.buffer[read_index];

        let result = self.gain * sample + self.ff * delayed + self.fb * delayed;
        self.buffer[self.write_index] = result;
        self.write_index = read_index;

        result
    }
}

pub fn new_ap(gain: f32, delay_samples: u32) -> UCF {
    UCF::new(gain, 1.0, -1.0 * gain, delay_samples)
}

pub fn new_ffcf(gain: f32, ff: f32) -> UCF {
    UCF::new(gain, ff, 0.0, 0)
}

pub fn new_fbcf(gain: f32, fb: f32) -> UCF {
    UCF::new(gain, 0.0, -1.0 * fb, 0)
}

pub fn new_delay_line(delay_samples: u32) -> UCF {
    UCF::new(0.0, 0.0, 0.0, delay_samples)
}
