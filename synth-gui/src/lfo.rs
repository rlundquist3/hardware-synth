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

pub struct LfoLayout {
    amp: f32,
    freq: f32,
    display_area: Rectangle,
}

impl LfoLayout {
    pub fn new(parameters: Vec<Parameter>, display_area: Rectangle) -> Self {
        LfoLayout {
            amp: parameters[0].get_value(),
            freq: parameters[1].get_value(),
            display_area,
        }
    }
}

impl Drawable for LfoLayout {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let stuff = Line::new(Point { x: 0, y: 0 }, Point { x: 0, y: 80 })
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));
        let footer = FooterMenu::new(["main", "lfo", "env", "filt", "fx"], 1);

        LinearLayout::vertical(Chain::new(stuff).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
