use alloc::boxed::Box;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Polyline, Primitive, Rectangle},
    text::Text,
};
use embedded_layout::{
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
};
use synth_core::effects::{Effect, filters::FILTER_COUNT};

use crate::{
    footer::FooterMenu,
    shared::{EMPTY_STYLE, LINE_STYLE, SELECTED_OPTION_STYLE, SMALL_TEXT_STYLE},
};

pub struct FiltersMainLayout<'a> {
    filters: &'a mut heapless::Vec<Box<dyn Effect>, FILTER_COUNT>,
    display_area: Rectangle,
    navigation_location: usize,
}

impl<'a> FiltersMainLayout<'a> {
    pub fn new(
        filters: &'a mut heapless::Vec<Box<dyn Effect>, FILTER_COUNT>,
        display_area: Rectangle,
        navigation_location: usize,
    ) -> Self {
        FiltersMainLayout {
            filters,
            display_area,
            navigation_location,
        }
    }
}

impl<'a> Drawable for FiltersMainLayout<'a> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let container_width: u32 = 24;
        let container_height: u32 = 24;
        let diagram_width: i32 = 20;
        let diagram_height: i32 = 20;

        let none_container = Rectangle::new(
            Point::zero(),
            Size {
                width: container_width,
                height: container_height,
            },
        )
        .into_styled(match self.navigation_location {
            1 => SELECTED_OPTION_STYLE,
            _ => EMPTY_STYLE,
        });
        let none_text = Text::new("none", Point::zero(), SMALL_TEXT_STYLE);
        let none_option = Chain::new(none_container).append(none_text.align_to(
            &none_container,
            horizontal::Center,
            vertical::Center,
        ));

        let lp_container = Rectangle::new(
            Point::zero(),
            Size {
                width: container_width,
                height: container_height,
            },
        )
        .into_styled(match self.navigation_location {
            2 => SELECTED_OPTION_STYLE,
            _ => EMPTY_STYLE,
        });

        let lp_points = [
            Point { x: 0, y: 0 },
            Point {
                x: diagram_width / 3,
                y: 0,
            },
            Point {
                x: diagram_width / 2,
                y: diagram_height,
            },
            Point {
                x: diagram_width,
                y: diagram_height,
            },
        ];
        let lp_curve = Polyline::new(&lp_points).into_styled(LINE_STYLE);
        let lp_diagram = Chain::new(lp_container).append(lp_curve.align_to(
            &lp_container,
            horizontal::Center,
            vertical::Center,
        ));

        let bp_container = Rectangle::new(
            Point::zero(),
            Size {
                width: container_width,
                height: container_height,
            },
        )
        .into_styled(match self.navigation_location {
            3 => SELECTED_OPTION_STYLE,
            _ => EMPTY_STYLE,
        });
        let bp_points = [
            Point {
                x: 0,
                y: diagram_height,
            },
            Point {
                x: diagram_width / 5,
                y: diagram_height,
            },
            Point {
                x: diagram_width / 3,
                y: 0,
            },
            Point {
                x: 2 * diagram_width / 3,
                y: 0,
            },
            Point {
                x: 4 * diagram_width / 5,
                y: diagram_height,
            },
            Point {
                x: diagram_width,
                y: diagram_height,
            },
        ];
        let bp_curve = Polyline::new(&bp_points).into_styled(LINE_STYLE);
        let bp_diagram = Chain::new(bp_container).append(bp_curve.align_to(
            &bp_container,
            horizontal::Center,
            vertical::Center,
        ));

        let hp_container = Rectangle::new(
            Point::zero(),
            Size {
                width: container_width,
                height: container_height,
            },
        )
        .into_styled(match self.navigation_location {
            4 => SELECTED_OPTION_STYLE,
            _ => EMPTY_STYLE,
        });
        let hp_points = [
            Point {
                x: 0,
                y: diagram_height,
            },
            Point {
                x: diagram_width / 2,
                y: diagram_height,
            },
            Point {
                x: 2 * diagram_width / 3,
                y: 0,
            },
            Point {
                x: diagram_width,
                y: 0,
            },
        ];
        let hp_curve = Polyline::new(&hp_points).into_styled(LINE_STYLE);
        let hp_diagram = Chain::new(hp_container).append(hp_curve.align_to(
            &hp_container,
            horizontal::Center,
            vertical::Center,
        ));

        let filter_options = LinearLayout::horizontal(
            Chain::new(none_option)
                .append(lp_diagram)
                .append(bp_diagram)
                .append(hp_diagram),
        )
        .with_alignment(vertical::Center)
        .with_spacing(spacing::DistributeFill(120))
        .arrange();

        let footer = FooterMenu::new(["main", "lfo", "env", "fltr", "fx"], 3);
        LinearLayout::vertical(Chain::new(filter_options).append(footer))
            .with_alignment(horizontal::Center)
            .with_spacing(spacing::DistributeFill(60))
            .arrange()
            .align_to(&self.display_area, horizontal::Center, vertical::Center)
            .draw(target)
    }
}
