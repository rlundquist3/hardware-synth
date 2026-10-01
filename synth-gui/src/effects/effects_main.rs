use alloc::boxed::Box;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Circle, Primitive, Rectangle},
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
use synth_core::effects::{EFFECT_COUNT, Effect};

use crate::{
    footer::FooterMenu,
    shared::{EMPTY_STYLE, FILLED_STYLE, SELECTED_OPTION_STYLE, SMALL_TEXT_STYLE},
};

pub struct EffectsMainLayout<'a> {
    effects: &'a mut heapless::Vec<Box<dyn Effect>, EFFECT_COUNT>,
    display_area: Rectangle,
    navigation_location: usize,
}

impl<'a> EffectsMainLayout<'a> {
    pub fn new(
        effects: &'a mut heapless::Vec<Box<dyn Effect>, EFFECT_COUNT>,
        display_area: Rectangle,
        navigation_location: usize,
    ) -> Self {
        EffectsMainLayout {
            effects,
            display_area,
            navigation_location,
        }
    }
}

impl<'a> Drawable for EffectsMainLayout<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut items: Vec<EffectItem, EFFECT_COUNT> = self
            .effects
            .iter()
            .enumerate()
            .map(|(i, e)| {
                EffectItem::new(e.get_name(), self.navigation_location == i + 1, e.is_on())
            })
            .collect();

        // TODO: scrolling list
        let list = LinearLayout::vertical(Views::new(&mut items))
            .with_alignment(horizontal::Left)
            .arrange();

        let footer = FooterMenu::new(["main", "lfo", "env", "fltr", "fx"], 4);

        LinearLayout::vertical(Chain::new(list).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}

pub struct EffectItem<'b> {
    name: &'b str,
    selected: bool,
    on: bool,
    position: Point,
}

impl<'b> EffectItem<'b> {
    pub fn new(name: &'b str, selected: bool, on: bool) -> Self {
        EffectItem {
            name,
            selected,
            on,
            position: Point::zero(),
        }
    }

    fn arrange(&self) -> impl View + Drawable<Color = BinaryColor, Output = ()> {
        let container = Rectangle::new(
            Point::zero(),
            Size {
                width: 40,
                height: 12,
            },
        )
        .into_styled(match self.selected {
            true => SELECTED_OPTION_STYLE,
            false => EMPTY_STYLE,
        });

        let text = Text::new(self.name, Point::zero(), SMALL_TEXT_STYLE);
        let toggle = Circle::new(Point::zero(), 4).into_styled(match self.on {
            true => FILLED_STYLE,
            false => EMPTY_STYLE,
        });

        Chain::new(container)
            .append(text.align_to(&container, horizontal::Left, vertical::Center))
            .append(toggle.align_to(&container, horizontal::Right, vertical::Center))
            .translate(self.position)
    }
}

impl<'b> View for EffectItem<'b> {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl<'b> Drawable for EffectItem<'b> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}
