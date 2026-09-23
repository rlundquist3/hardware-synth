use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_4X6},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::Text,
};
use embedded_layout::prelude::*;

type ComposedButton<'a> = Link<
    Text<'a, MonoTextStyle<'a, BinaryColor>>,
    Chain<embedded_graphics::primitives::Styled<RoundedRectangle, PrimitiveStyle<BinaryColor>>>,
>;
pub struct Button<'a> {
    text: &'a str,
    active: bool,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Button {
            text,
            active: false,
        }
    }

    pub fn format(self) -> ComposedButton<'a> {
        let button_style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(BinaryColor::On)
            .build();
        let button_text_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);

        let container = RoundedRectangle::with_equal_corners(
            Rectangle::new(
                Point::zero(),
                Size {
                    width: 20,
                    height: 10,
                },
            ),
            Size {
                width: 4,
                height: 4,
            },
        )
        .into_styled(button_style);
        let text = Text::new(self.text, Point::zero(), button_text_style);

        Chain::new(container).append(text.align_to(
            &container,
            horizontal::Center,
            vertical::Center,
        ))
    }
}
