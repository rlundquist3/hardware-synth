use core::fmt::Write;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, primitives::Rectangle, text::Text};
use embedded_layout::{
    layout::linear::{LinearLayout, spacing},
    prelude::*,
};
use heapless::String;
use synth_core::parameter::Parameter;

use crate::{footer::FooterMenu, shared::BASE_TEXT_STYLE};

pub struct DualLayout {
    waveform_a: String<8>,
    offset_a: String<8>,
    waveform_b: String<8>,
    offset_b: String<8>,
    display_area: Rectangle,
}

impl DualLayout {
    pub fn new(parameters: &[Parameter], display_area: Rectangle) -> Self {
        let mut waveform_a: String<8> = String::new();
        let mut offset_a: String<8> = String::new();
        let mut waveform_b: String<8> = String::new();
        let mut offset_b: String<8> = String::new();
        write!(
            waveform_a,
            "{}",
            match parameters[0].get_value() {
                1.0 => "Sine",
                2.0 => "Saw",
                3.0 => "Sqr",
                4.0 => "Tri",
                _ => "",
            }
        )
        .unwrap();
        write!(offset_a, "{:.3}", parameters[1].get_value()).unwrap();
        write!(
            waveform_b,
            "{}",
            match parameters[2].get_value() {
                1.0 => "Sine",
                2.0 => "Saw",
                3.0 => "Sqr",
                4.0 => "Tri",
                _ => "",
            }
        )
        .unwrap();
        write!(offset_b, "{:.3}", parameters[3].get_value()).unwrap();

        DualLayout {
            waveform_a,
            offset_a,
            waveform_b,
            offset_b,
            display_area,
        }
    }
}

impl Drawable for DualLayout {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // TODO: visualize offsets, or at least format more nicely
        let wave_a_text = Text::new(&self.waveform_a, Point::zero(), BASE_TEXT_STYLE);
        let offset_a_text = Text::new(&self.offset_a, Point::zero(), BASE_TEXT_STYLE);

        let wave_b_text = Text::new(&self.waveform_b, Point::zero(), BASE_TEXT_STYLE);
        let offset_b_text = Text::new(&self.offset_b, Point::zero(), BASE_TEXT_STYLE);

        let a = LinearLayout::vertical(Chain::new(wave_a_text).append(offset_a_text))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::Tight)
            .arrange();
        let b = LinearLayout::vertical(Chain::new(wave_b_text).append(offset_b_text))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::Tight)
            .arrange();

        let engine_params = LinearLayout::horizontal(Chain::new(a).append(b))
            .with_alignment(vertical::Center)
            .with_spacing(spacing::DistributeFill(80))
            .arrange();
        let footer = FooterMenu::new(["<-", "main", "lfo", "env", "fltr", "fx"], 1);

        LinearLayout::vertical(Chain::new(engine_params).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
