use alloc::{collections::VecDeque, format, string::String};
use core::cell::RefCell;
use embassy_sync::{
    blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex},
    channel::Channel,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X10, FONT_9X18_BOLD},
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle},
    text::{Alignment, Text},
};
use synth_core::{
    engines::fm::{FMSynth, MOD_INDEX_RENDER},
    parameter::UserParameters,
};

use crate::display::{Display, DisplayError};

pub async fn render_engine_main(
    display: &mut Display,
    engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
) -> Result<(), DisplayError> {
    let ratio_text_style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::On);

    let mut c = 0.0;
    let mut m = 0.0;
    let mut i = 0.0;
    engine.lock(|e| {
        let engine = e.borrow();

        let params = engine.get_parameters();
        c = params[0].get_value();
        m = params[1].get_value();
        i = params[2].get_value();
    });

    Text::with_alignment(
        &format!("{:0}", c),
        Point { x: 32, y: 28 },
        ratio_text_style,
        Alignment::Center,
    )
    .draw(display)?;
    Line::new(Point { x: 20, y: 32 }, Point { x: 44, y: 32 })
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
        .draw(display)?;
    Text::with_alignment(
        &format!("{:0}", m),
        Point { x: 32, y: 44 },
        ratio_text_style,
        Alignment::Center,
    )
    .draw(display)?;
    Text::with_alignment(
        MOD_INDEX_RENDER[i as usize],
        Point { x: 80, y: 36 },
        ratio_text_style,
        Alignment::Center,
    )
    .draw(display)?;

    Ok(())
}
