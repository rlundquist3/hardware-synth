use crate::buffer_pool::BufferPool;
use crate::effects::EffectComponent;

// Adapted from https://ccrma.stanford.edu/~jos/pasp/Lowpass_Feedback_Comb_Filter.html

#[derive(Debug)]
pub struct LBCF {
    feedback: f32,
    damping: f32,
    buffer: &'static mut [f32],
    write_index: usize,
    filter_state: f32,
}

impl LBCF {
    pub fn new(
        feedback: f32,
        damping: f32,
        delay_samples: u32,
        buffer_pool: &mut BufferPool,
    ) -> Self {
        LBCF {
            feedback,
            damping,
            buffer: buffer_pool.take(delay_samples as usize),
            write_index: 0,
            filter_state: 0.0,
        }
    }
}

impl EffectComponent for LBCF {
    fn process(&mut self, sample: f32) -> f32 {
        let delayed = self.buffer[self.write_index];

        self.filter_state = (1.0 - self.damping) * delayed + self.damping * self.filter_state;
        self.buffer[self.write_index] = sample + self.feedback * self.filter_state;
        self.write_index = (self.write_index + 1) % self.buffer.len();

        delayed
    }
}
