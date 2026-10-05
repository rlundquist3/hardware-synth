use crate::buffer_pool::BufferPool;
use crate::effects::EffectComponent;

const MAX_DELAY_SECS: f32 = 1.0;

#[derive(Debug)]
pub struct UCF {
    buffer: &'static mut [f32],
    write_index: usize,
    gain: f32,
    ff: f32,
    fb: f32,
}

impl UCF {
    pub fn new(
        gain: f32,
        ff: f32,
        fb: f32,
        delay_samples: u32,
        buffer_pool: &mut BufferPool,
    ) -> Self {
        UCF {
            buffer: buffer_pool.take(delay_samples as usize),
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

pub fn new_ap(gain: f32, delay_samples: u32, buffer_pool: &mut BufferPool) -> UCF {
    UCF::new(gain, 1.0, -1.0 * gain, delay_samples, buffer_pool)
}

pub fn new_ffcf(gain: f32, ff: f32, buffer_pool: &mut BufferPool) -> UCF {
    UCF::new(gain, ff, 0.0, 0, buffer_pool)
}

pub fn new_fbcf(gain: f32, fb: f32, buffer_pool: &mut BufferPool) -> UCF {
    UCF::new(gain, 0.0, -1.0 * fb, 0, buffer_pool)
}

pub fn new_delay_line(delay_samples: u32, buffer_pool: &mut BufferPool) -> UCF {
    UCF::new(0.0, 0.0, 0.0, delay_samples, buffer_pool)
}
