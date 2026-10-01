use alloc::boxed::Box;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Line, Primitive, Rectangle},
};
use embedded_layout::{
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
};
use synth_core::effects::{
    Effect,
    filters::{FILTER_COUNT, biquad::Q_RANGE},
};

use crate::{
    effects::{
        controls::Dial,
        filters_main::{FilterDiagram, FilterTypes},
    },
    footer::FooterMenu,
    shared::{EMPTY_STYLE, LINE_STYLE},
};

pub struct FiltersDetailLayout<'a> {
    filters: &'a mut heapless::Vec<Box<dyn Effect>, FILTER_COUNT>,
    display_area: Rectangle,
    navigation_location: usize,
}

impl<'a> FiltersDetailLayout<'a> {
    pub fn new(
        filters: &'a mut heapless::Vec<Box<dyn Effect>, FILTER_COUNT>,
        display_area: Rectangle,
        navigation_location: usize,
    ) -> Self {
        FiltersDetailLayout {
            filters,
            display_area,
            navigation_location,
        }
    }
}

impl<'a> Drawable for FiltersDetailLayout<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let filter_index = self.navigation_location - 1;
        let filter = &self.filters[filter_index];
        let on = filter.is_on();
        let params = filter.get_parameters();
        let cutoff_freq = params[0].get_value();
        let q = params[1].get_value();
        let filter_type = match self.navigation_location {
            1 => FilterTypes::LowPass,
            2 => FilterTypes::BandPass,
            3 => FilterTypes::HighPass,
            _ => FilterTypes::LowPass,
        };

        let container_width: u32 = 84;
        let container_height: u32 = 44;
        let diagram_width: i32 = 80;
        let diagram_height: i32 = 32;

        let diagram = FilterDiagram::new(
            filter_type,
            container_width,
            container_height,
            diagram_width,
            diagram_height,
            cutoff_freq,
            false,
            on,
        );

        let q_dial = Dial::new("Q", Q_RANGE, q);

        let parameters = LinearLayout::horizontal(Chain::new(diagram).append(q_dial))
            .with_alignment(vertical::Center)
            .with_spacing(spacing::DistributeFill(120))
            .arrange();

        let footer = FooterMenu::new(["back", "", "", "", ""], 0);
        LinearLayout::vertical(Chain::new(parameters).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
