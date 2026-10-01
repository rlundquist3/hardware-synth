use alloc::boxed::Box;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Primitive, Rectangle},
    text::Text,
};
use embedded_layout::{
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
    view_group::Views,
};
use heapless::Vec;
use synth_core::effects::{EFFECT_COUNT, Effect};

use crate::{
    effects::controls::{Dial, Toggle},
    footer::FooterMenu,
    shared::{FILLED_STYLE, SELECTED_OPTION_STYLE, SMALL_TEXT_STYLE},
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

        let toggle = Toggle::new(on);

        let mut items: Vec<Dial, 4> = params
            .iter()
            .map(|p| Dial::new(p.get_name(), p.get_range(), p.get_value()))
            .collect();

        let controls = LinearLayout::horizontal(Chain::new(toggle).append(Views::new(&mut items)))
            .with_alignment(vertical::Center)
            .with_spacing(spacing::FixedMargin(12))
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
