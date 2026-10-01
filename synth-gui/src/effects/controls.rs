use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Arc, Primitive, Rectangle},
    text::Text,
};
use embedded_layout::{
    View,
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
};

use crate::shared::{
    EMPTY_STYLE, FILLED_STYLE, LINE_STYLE, SELECTED_OPTION_STYLE, SMALL_TEXT_STYLE,
};

pub struct Toggle {
    on: bool,
    position: Point,
}

impl Toggle {
    pub fn new(on: bool) -> Self {
        Toggle {
            on,
            position: Point::zero(),
        }
    }

    fn arrange(&self) -> impl View + Drawable<Color = BinaryColor, Output = ()> {
        LinearLayout::vertical(
            Chain::new(
                Rectangle::new(
                    Point::zero(),
                    Size {
                        width: 4,
                        height: 6,
                    },
                )
                .into_styled(match self.on {
                    true => FILLED_STYLE,
                    false => SELECTED_OPTION_STYLE,
                }),
            )
            .append(
                Rectangle::new(
                    Point::zero(),
                    Size {
                        width: 4,
                        height: 6,
                    },
                )
                .into_styled(match self.on {
                    false => FILLED_STYLE,
                    true => SELECTED_OPTION_STYLE,
                }),
            ),
        )
        .with_alignment(horizontal::Center)
        .with_spacing(spacing::Tight)
        .arrange()
        .translate(self.position)
    }
}

impl View for Toggle {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl Drawable for Toggle {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}

pub struct Dial<'a> {
    name: &'a str,
    range: (f32, f32),
    value: f32,
    position: Point,
}

impl<'a> Dial<'a> {
    pub fn new(name: &'a str, range: (f32, f32), value: f32) -> Self {
        Dial {
            name,
            range,
            value,
            position: Point::zero(),
        }
    }

    fn arrange(&self) -> impl View + Drawable<Color = BinaryColor, Output = ()> {
        let container = Rectangle::new(
            Point::zero(),
            Size {
                width: 20,
                height: 20,
            },
        )
        .into_styled(EMPTY_STYLE);

        let angle_sweep =
            ((self.value - self.range.0) / (self.range.1 - self.range.0) * 270.0).deg();

        let text = Text::new(self.name, Point::zero(), SMALL_TEXT_STYLE);
        let dial = Arc::new(Point::zero(), 12, 135.0.deg(), angle_sweep).into_styled(LINE_STYLE);

        Chain::new(container)
            .append(text.align_to(&container, horizontal::Center, vertical::Bottom))
            .append(dial.align_to(&container, horizontal::Center, vertical::Top))
            .translate(self.position)
    }
}

impl<'a> View for Dial<'a> {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl<'a> Drawable for Dial<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}
