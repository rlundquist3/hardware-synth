use crate::{
    SAMPLE_RATE,
    buffer_pool::BufferPool,
    effects::EffectComponent,
    oscillator::{Oscillator, Waveform::Sine},
    parameter::Parameter,
};
use micromath::F32Ext;

#[derive(Debug)]
pub struct LFODelay {
    lfo: Oscillator,
    lfo_amp: Parameter,
    lfo_freq: Parameter,
    delay_ms: Parameter,
    buffer: &'static mut [f32],
    write_index: usize,
}

const MAX_DELAY_SECS: f32 = 100.0 / 1000.0;

impl LFODelay {
    pub fn new(
        delay_ms: Parameter,
        amp: Parameter,
        freq: Parameter,
        buffer_pool: &mut BufferPool,
    ) -> Self {
        let mut lfo = Oscillator::new(Sine);
        lfo.set_freq(freq.get_value());
        let max_samples = (MAX_DELAY_SECS * SAMPLE_RATE as f32) as usize;

        LFODelay {
            lfo,
            delay_ms,
            lfo_amp: amp,
            lfo_freq: freq,
            buffer: buffer_pool.take(max_samples),
            write_index: 0,
        }
    }
}

impl EffectComponent for LFODelay {
    fn process(&mut self, sample: f32) -> f32 {
        let freq_param = self.lfo_freq.get_value();
        if self.lfo.freq != freq_param {
            self.lfo.set_freq(freq_param);
        }

        let raw_delay_samples = self.delay_ms.get_value() / 1000.0 * SAMPLE_RATE as f32;
        let lfo_sample = self.lfo.next_sample();

        let delay_samples_mod = lfo_sample * self.lfo_amp.get_value() * raw_delay_samples;
        let delay_samples_float =
            (raw_delay_samples + delay_samples_mod).clamp(1.0, self.buffer.len() as f32 - 2.0);

        let mut read_index_float = self.write_index as f32 - delay_samples_float;
        if read_index_float < 0.0 {
            read_index_float += self.buffer.len() as f32;
        }

        // linear interpolation of delayed samples
        let i_0 = read_index_float.floor() as usize % self.buffer.len();
        let i_1 = (i_0 + 1) % self.buffer.len();
        let f = read_index_float - i_0 as f32;
        let delayed = (1.0 - f) * self.buffer[i_0] + f * self.buffer[i_1];

        self.buffer[self.write_index] = sample;
        self.write_index = (self.write_index + 1) % self.buffer.len();

        delayed
    }
}
