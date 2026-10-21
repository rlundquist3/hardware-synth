use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use embedded_graphics::prelude::*;
use ssd1306::{Ssd1306, prelude::*};

use crate::{
    SharedChain,
    controls::{
        MODE,
        Mode::{
            EffectsDetail, EffectsList, EngineEnvelope, EngineLFO, EngineList, EngineMain,
            FiltersDetail, FiltersList,
        },
    },
    display::{
        effects::{render_effects_detail, render_effects_list},
        engine::{
            render_engine_envelope, render_engine_lfo, render_engine_list, render_engine_main,
        },
        filters::{render_filters_detail, render_filters_list},
    },
    i2c::SharedI2c,
};

mod effects;
mod engine;
mod filters;

pub const DISPLAY_ADDRESS: u8 = 0x3D;

pub type Display = Ssd1306<
    I2CInterface<SharedI2c>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;
pub type DisplayError = <Display as DrawTarget>::Error;

pub static DISPLAY_BUFFER: Channel<CriticalSectionRawMutex, u32, 16> = Channel::new();

#[embassy_executor::task]
pub async fn display_handler(mut display: Display, chain: &'static SharedChain) {
    let mut mode_rx = MODE.receiver().unwrap();

    loop {
        let _content = DISPLAY_BUFFER.receive().await;
        display.clear_buffer();

        let mode = mode_rx.get().await;
        match mode {
            EngineList => render_engine_list(&mut display, chain).await.unwrap(),
            EngineMain => render_engine_main(&mut display, chain).await.unwrap(),
            EngineLFO => render_engine_lfo(&mut display, chain).await.unwrap(),
            EngineEnvelope => render_engine_envelope(&mut display, chain).await.unwrap(),
            FiltersList => render_filters_list(&mut display, chain).await.unwrap(),
            FiltersDetail => render_filters_detail(&mut display, chain).await.unwrap(),
            EffectsList => render_effects_list(&mut display, chain).await.unwrap(),
            EffectsDetail => render_effects_detail(&mut display, chain).await.unwrap(),
        }

        if display.flush().is_err() {
            Timer::after_millis(2).await;
            display.flush().ok();
        }
    }
}
