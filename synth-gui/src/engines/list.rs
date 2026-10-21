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
use synth_core::engines::ENGINE_COUNT;

use crate::{footer::FooterMenu, shared::ListItem};

pub const ENGINE_CHUNK_SIZE: usize = 5;
pub const ENGINE_CHUNK_COUNT: usize = ENGINE_COUNT / ENGINE_CHUNK_SIZE
    + match ENGINE_COUNT % ENGINE_CHUNK_SIZE {
        0 => 0,
        _ => 1,
    };

pub struct EngineListLayout<'a> {
    items: [&'a str; ENGINE_COUNT],
    current_engine_index: usize,
    display_area: Rectangle,
    navigation_location: usize,
}

impl<'a> EngineListLayout<'a> {
    pub fn new(
        items: [&'a str; ENGINE_COUNT],
        current_engine_index: usize,
        display_area: Rectangle,
        navigation_location: usize,
    ) -> Self {
        EngineListLayout {
            items,
            current_engine_index,
            display_area,
            navigation_location,
        }
    }
}

impl<'a> Drawable for EngineListLayout<'a> {
    type Color = BinaryColor;
    type Output = ();

    // TODO: this layout will work for up to 15 items, once that threshold is crossed,
    // will need to implement scrolling
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut items: Vec<ListItem, ENGINE_COUNT> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, name)| {
                ListItem::new(
                    name,
                    self.navigation_location == i + 1,
                    i == self.current_engine_index,
                )
            })
            .collect();

        let mut column_layouts: Vec<_, ENGINE_CHUNK_COUNT> = items
            .chunks_mut(ENGINE_CHUNK_SIZE)
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

        let footer = FooterMenu::new(["", "", "", "", "", "->"], 5);

        LinearLayout::vertical(Chain::new(columns).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
