use alloc::vec::Vec;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
};
use embedded_layout::{
    layout::linear::{LinearLayout, spacing},
    prelude::*,
};
use synth_core::parameter::Parameter;

use crate::footer::FooterMenu;

pub struct EnvelopeLayout {
    a: f32,
    d: f32,
    s: f32,
    r: f32,
    display_area: Rectangle,
}

impl EnvelopeLayout {
    pub fn new(parameters: Vec<Parameter>, display_area: Rectangle) -> Self {
        EnvelopeLayout {
            a: parameters[0].get_value(),
            d: parameters[1].get_value(),
            s: parameters[2].get_value(),
            r: parameters[3].get_value(),
            display_area,
        }
    }
}

impl Drawable for EnvelopeLayout {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        let width: i32 = 100;
        let height: i32 = 48;
        let start_point = Point { x: 0, y: height };
        let attack_point = Point {
            x: (self.a / 2.0 * width as f32 / 3.0) as i32,
            y: 0,
        };
        let decay_point = Point {
            x: attack_point.x + (self.d / 2.0 * width as f32 / 3.0) as i32,
            y: ((1.0 - self.s) * height as f32) as i32,
        };
        let release_point = Point {
            x: width - (self.r / 2.0 * width as f32 / 3.0) as i32,
            y: ((1.0 - self.s) * height as f32) as i32,
        };
        let end_point = Point {
            x: width,
            y: height,
        };

        let envelope = Chain::new(Line::new(start_point, attack_point).into_styled(line_style))
            .append(Line::new(attack_point, decay_point).into_styled(line_style))
            .append(Line::new(decay_point, release_point).into_styled(line_style))
            .append(Line::new(release_point, end_point).into_styled(line_style));

        let footer = FooterMenu::new(["main", "lfo", "env", "filt", "fx"], 2);

        LinearLayout::vertical(Chain::new(envelope).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
