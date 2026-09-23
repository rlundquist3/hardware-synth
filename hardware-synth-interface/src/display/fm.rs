use alloc::{format, vec::Vec};
use core::cell::RefCell;
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X18_BOLD},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Alignment, Text},
};
use synth_core::{
    engines::fm::{FMSynth, MOD_INDEX_RENDER},
    parameter::{Parameter, UserParameters},
};
use synth_gui::engines::fm::EngineMainLayout;

use crate::display::{Display, DisplayError};

pub async fn render_engine_main(
    display: &mut Display,
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let mut parameters: Vec<Parameter> = Vec::new();
    engine.lock(|e| {
        let engine = e.borrow();

        parameters = engine.get_parameters();
    });

    EngineMainLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}
