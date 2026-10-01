use alloc::boxed::Box;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Arc, Circle, Line, Primitive, Rectangle},
    text::Text,
};
use embedded_layout::{
    View,
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
    view_group::Views,
};
use heapless::Vec;
use micromath::F32Ext;
use synth_core::{
    effects::{EFFECT_COUNT, Effect},
    parameter,
};

use crate::{
    footer::FooterMenu,
    shared::{EMPTY_STYLE, FILLED_STYLE, LINE_STYLE, SELECTED_OPTION_STYLE, SMALL_TEXT_STYLE},
};

pub struct EffectsDetailLayout<'a> {
    effects: &'a mut heapless::Vec<Box<dyn Effect>, EFFECT_COUNT>,
    display_area: Rectangle,
    navigation_location: usize,
}

impl<'a> EffectsDetailLayout<'a> {
    pub fn new(
        effects: &'a mut heapless::Vec<Box<dyn Effect>, EFFECT_COUNT>,
        display_area: Rectangle,
        navigation_location: usize,
    ) -> Self {
        EffectsDetailLayout {
            effects,
            display_area,
            navigation_location,
        }
    }
}

impl<'a> Drawable for EffectsDetailLayout<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let effect_index = self.navigation_location - 1;
        let effect = &self.effects[effect_index];
        let on = effect.is_on();
        let params = effect.get_parameters();

        let toggle = LinearLayout::vertical(
            Chain::new(
                Rectangle::new(
                    Point::zero(),
                    Size {
                        width: 4,
                        height: 6,
                    },
                )
                .into_styled(match on {
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
                .into_styled(match on {
                    false => FILLED_STYLE,
                    true => SELECTED_OPTION_STYLE,
                }),
            ),
        )
        .with_alignment(horizontal::Center)
        .arrange();

        let mut items: Vec<Dial, 4> = params
            .iter()
            .map(|p| Dial::new(p.get_name(), p.get_range(), p.get_value()))
            .collect();

        let controls = LinearLayout::horizontal(Chain::new(toggle).append(Views::new(&mut items)))
            .with_alignment(vertical::Center)
            .with_spacing(spacing::DistributeFill(100))
            .arrange();

        let text = Text::new(effect.get_name(), Point::zero(), SMALL_TEXT_STYLE);
        let footer = FooterMenu::new(["back", "", "", "", ""], 0);

        LinearLayout::vertical(Chain::new(text).append(controls).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}

pub struct Dial<'b> {
    name: &'b str,
    range: (f32, f32),
    value: f32,
    position: Point,
}

impl<'b> Dial<'b> {
    pub fn new(name: &'b str, range: (f32, f32), value: f32) -> Self {
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

impl<'b> View for Dial<'b> {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl<'b> Drawable for Dial<'b> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}
