use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_4X6},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_layout::{
    layout::linear::{LinearLayout, spacing},
    prelude::*,
};

use crate::shared::SMALL_TEXT_STYLE;

type ComposedItem<'a> = Link<
    Text<'a, MonoTextStyle<'a, BinaryColor>>,
    Chain<embedded_graphics::primitives::Styled<Rectangle, PrimitiveStyle<BinaryColor>>>,
>;
pub struct Item<'a> {
    text: &'a str,
    active: bool,
}

impl<'a> Item<'a> {
    pub fn new(text: &'a str, active: bool) -> Self {
        Item { text, active }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn format(self) -> ComposedItem<'a> {
        let button_style = PrimitiveStyleBuilder::new()
            .stroke_width(match self.active {
                true => 2,
                false => 1,
            })
            .stroke_color(BinaryColor::On)
            .build();

        let container = Rectangle::new(
            Point::zero(),
            Size {
                width: 25,
                height: 10,
            },
        )
        .into_styled(button_style);
        let text = Text::new(self.text, Point::zero(), SMALL_TEXT_STYLE);

        Chain::new(container).append(text.align_to(
            &container,
            horizontal::Center,
            vertical::Center,
        ))
    }
}

pub struct FooterMenu<'a> {
    names: [&'a str; 5],
    active_index: usize,
    position: Point,
}

impl<'a> FooterMenu<'a> {
    pub fn new(names: [&'a str; 5], active_index: usize) -> Self {
        FooterMenu {
            names,
            active_index,
            position: Point::zero(),
        }
    }

    fn arrange(&self) -> impl View + Drawable<Color = BinaryColor, Output = ()> {
        LinearLayout::horizontal(
            Chain::new(Item::new(self.names[0], self.active_index == 0).format())
                .append(Item::new(self.names[1], self.active_index == 1).format())
                .append(Item::new(self.names[2], self.active_index == 2).format())
                .append(Item::new(self.names[3], self.active_index == 3).format())
                .append(Item::new(self.names[4], self.active_index == 4).format()),
        )
        .with_spacing(spacing::Tight)
        .arrange()
        .translate(self.position)
    }
}

impl<'a> View for FooterMenu<'a> {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl<'a> Drawable for FooterMenu<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}
