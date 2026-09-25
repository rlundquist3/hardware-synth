use core::cell::RefCell;

use daisy_embassy::audio::{HALF_DMA_BUFFER_LENGTH, Interface, Running};
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use synth_core::{chain::Chain, engines::fm::FMSynth, utils::f32_to_sample};

use crate::SharedChain;

#[embassy_executor::task]
pub async fn audio_handler(
    mut interface: Interface<'static, Running>,
    chain: &'static SharedChain,
) {
    interface
        .start_callback(|_input, output| {
            chain.lock(|c: &RefCell<Chain>| {
                audio_output(&mut c.borrow_mut(), output);
            });
        })
        .await
        .unwrap();
}

fn audio_output(chain: &mut Chain, output: &mut [u32]) {
    let mut buf = [0; HALF_DMA_BUFFER_LENGTH];

    buf.chunks_mut(2).for_each(|chunk| {
        let sample = f32_to_sample(chain.next().unwrap_or(0.0));
        chunk[0] = sample;
        chunk[1] = sample;
    });

    output.copy_from_slice(&buf);
}
