use core::mem::take;

pub struct BufferPool {
    remaining: &'static mut [f32],
}

impl BufferPool {
    pub fn from_slice(backing: &'static mut [f32]) -> Self {
        BufferPool { remaining: backing }
    }

    pub fn take(&mut self, len: usize) -> &'static mut [f32] {
        let remaining = take(&mut self.remaining);
        let (chunk, rest) = remaining.split_at_mut(len);

        self.remaining = rest;
        chunk.fill(0.0);

        chunk
    }

    pub fn remaining(&self) -> usize {
        self.remaining.len()
    }
}
