use embedded_graphics::{
    Drawable, draw_target::DrawTarget, pixelcolor::BinaryColor, primitives::Rectangle,
};
use embedded_layout::{
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
    view_group::Views,
};
use heapless::Vec;
use synth_core::effects::{EFFECT_COUNT, EffectSnapshot};

use crate::{footer::FooterMenu, shared::ListItem};

pub const EFFECT_CHUNK_SIZE: usize = 5;
pub const EFFECT_CHUNK_COUNT: usize = EFFECT_COUNT / EFFECT_CHUNK_SIZE
    + match EFFECT_COUNT % EFFECT_CHUNK_SIZE {
        0 => 0,
        _ => 1,
    };

pub struct EffectsListLayout {
    effects: Vec<EffectSnapshot, EFFECT_COUNT>,
    display_area: Rectangle,
    navigation_location: usize,
}

impl EffectsListLayout {
    pub fn new(
        effects: Vec<EffectSnapshot, EFFECT_COUNT>,
        display_area: Rectangle,
        navigation_location: usize,
    ) -> Self {
        EffectsListLayout {
            effects,
            display_area,
            navigation_location,
        }
    }
}

impl Drawable for EffectsListLayout {
    type Color = BinaryColor;
    type Output = ();

    // TODO: this layout will work for up to 15 effects, once that threshold is crossed,
    // will need to implement scrolling
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut items: Vec<ListItem, EFFECT_COUNT> = self
            .effects
            .iter()
            .enumerate()
            .map(|(i, e)| ListItem::new(e.name, self.navigation_location == i + 1, e.on))
            .collect();

        let mut column_layouts: Vec<_, EFFECT_CHUNK_COUNT> = items
            .chunks_mut(EFFECT_CHUNK_SIZE)
            .map(|chunk| {
                LinearLayout::vertical(Views::new(chunk))
                    .with_alignment(horizontal::Left)
                    .arrange()
            })
            .collect();

        let columns = LinearLayout::horizontal(Views::new(&mut column_layouts))
            .with_alignment(vertical::Top)
            .with_spacing(spacing::DistributeFill(120))
            .arrange();

        let footer = FooterMenu::new(["<-", "main", "lfo", "env", "fltr", "fx"], 5);

        LinearLayout::vertical(Chain::new(columns).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
