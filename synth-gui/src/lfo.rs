use core::f32::consts::PI;

use alloc::vec::Vec;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Polyline, PrimitiveStyle, Rectangle},
};
use embedded_layout::{
    layout::linear::{LinearLayout, spacing},
    prelude::*,
};
use micromath::F32Ext;
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
        let width: i32 = 120;
        let height: i32 = 40;
        let plot_container = Rectangle::new(
            Point::zero(),
            Size {
                width: width as u32,
                height: height as u32,
            },
        )
        .into_styled(PrimitiveStyle::new());

        let mut points = Vec::new();
        for x in 0..(width) {
            let phase = 2.0 * PI * (x as f32 / width as f32);
            let y = self.amp * (self.freq * phase).sin();

            points.push(Point {
                x,
                y: (height as f32 / 10.0 * y) as i32,
            });
        }
        let curve =
            Polyline::new(&points).into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1));
        let plot = Chain::new(plot_container).append(curve.align_to(
            &plot_container,
            horizontal::Center,
            vertical::Center,
        ));

        let footer = FooterMenu::new(["main", "lfo", "env", "filt", "fx"], 1);

        LinearLayout::vertical(Chain::new(plot).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
