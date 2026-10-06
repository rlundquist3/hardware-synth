use core::fmt::Write;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_9X18_BOLD},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_layout::{
    layout::linear::{LinearLayout, spacing},
    prelude::*,
};
use heapless::String;
use synth_core::{engines::fm::MOD_INDEX_RENDER, parameter::Parameter};

use crate::{footer::FooterMenu, shared::SMALL_TEXT_STYLE};

pub struct EngineMainLayout {
    waveform: String<8>,
    c: String<8>,
    m: String<8>,
    i: &'static str,
    display_area: Rectangle,
}

impl EngineMainLayout {
    pub fn new(parameters: &[Parameter], display_area: Rectangle) -> Self {
        let mut waveform: String<8> = String::new();
        let mut c: String<8> = String::new();
        let mut m: String<8> = String::new();
        write!(
            waveform,
            "{}",
            match parameters[0].get_value() {
                1.0 => "Sine",
                2.0 => "Saw",
                3.0 => "Sqr",
                _ => "",
            }
        )
        .unwrap();
        write!(c, "{:.0}", parameters[1].get_value()).unwrap();
        write!(m, "{:.0}", parameters[2].get_value()).unwrap();

        EngineMainLayout {
            waveform,
            c,
            m,
            i: MOD_INDEX_RENDER[parameters[3].get_value() as usize],
            display_area,
        }
    }
}

impl Drawable for EngineMainLayout {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let wave_text = Text::new(&self.waveform, Point::zero(), SMALL_TEXT_STYLE);

        let ratio_text_style = MonoTextStyle::new(&FONT_9X18_BOLD, BinaryColor::On);

        let c_text = Text::new(&self.c, Point::zero(), ratio_text_style);
        let m_text = Text::new(&self.m, Point::zero(), ratio_text_style);
        let ratio_line = Line::new(Point { x: 0, y: 0 }, Point { x: 24, y: 0 })
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2));
        let i_text = Text::new(self.i, Point::zero(), ratio_text_style);

        let ratio = LinearLayout::vertical(Chain::new(c_text).append(ratio_line).append(m_text))
            .with_alignment(horizontal::Center)
            .arrange();
        let engine_params =
            LinearLayout::horizontal(Chain::new(wave_text).append(ratio).append(i_text))
                .with_alignment(vertical::Center)
                .with_spacing(spacing::DistributeFill(120))
                .arrange();
        let footer = FooterMenu::new(["main", "lfo", "env", "fltr", "fx"], 0);

        LinearLayout::vertical(Chain::new(engine_params).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
