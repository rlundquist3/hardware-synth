use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_4X6, FONT_6X9},
    },
    pixelcolor::BinaryColor,
    primitives::PrimitiveStyle,
};

pub const LINE_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

pub const EMPTY_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::new();
pub const SELECTED_OPTION_STYLE: PrimitiveStyle<BinaryColor> =
    PrimitiveStyle::with_stroke(BinaryColor::On, 1);

pub const BASE_TEXT_STYLE: MonoTextStyle<'_, BinaryColor> =
    MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
pub const SMALL_TEXT_STYLE: MonoTextStyle<'_, BinaryColor> =
    MonoTextStyle::new(&FONT_4X6, BinaryColor::On);
