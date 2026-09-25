use alloc::{collections::VecDeque, string::String};
use core::cell::RefCell;
use embassy_stm32::i2c::I2c;
use embassy_sync::{
    blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex},
    channel::Channel,
};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use ssd1306::{Ssd1306, prelude::*};
use synth_core::engines::fm::FMSynth;

use crate::{
    SharedChain,
    controls::{
        MODE,
        Mode::{
            EffectsDetail, EffectsMain, EngineEnvelope, EngineLFO, EngineMain, FilterDetail,
            FiltersMain,
        },
    },
    display::fm::render_engine_main,
};

mod fm;

pub type Display = Ssd1306<
    I2CInterface<I2c<'static, embassy_stm32::mode::Blocking, embassy_stm32::i2c::Master>>,
    DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>,
>;
pub type DisplayError = <Display as DrawTarget>::Error;

pub struct DisplayContent {
    // pub text: String,
}

pub static DISPLAY_BUFFER: Channel<CriticalSectionRawMutex, u32, 16> = Channel::new();

const TEXT_STYLE: MonoTextStyle<'_, BinaryColor> = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

#[embassy_executor::task]
pub async fn display_handler(mut display: Display, chain: &'static SharedChain) {
    let mut mode_rx = MODE.receiver().unwrap();

    loop {
        let content = DISPLAY_BUFFER.receive().await;
        display.clear_buffer();

        let mode = mode_rx.get().await;
        match mode {
            EngineMain => render_engine_main(&mut display, chain).await.unwrap(),
            EngineEnvelope => {}
            EngineLFO => {}
            FiltersMain => {}
            FilterDetail => {}
            EffectsMain => {}
            EffectsDetail => {}
        }

        display.flush().unwrap();
    }
}
