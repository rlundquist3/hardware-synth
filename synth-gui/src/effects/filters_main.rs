use alloc::boxed::Box;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{Line, Polyline, Primitive, Rectangle},
    text::Text,
};
use embedded_layout::{
    View,
    align::{Align, horizontal, vertical},
    layout::linear::{LinearLayout, spacing},
    object_chain::Chain,
};
use synth_core::effects::{
    Effect,
    filters::{
        FILTER_COUNT, band_pass::BP_CUTOFF_RANGE, biquad::Q_RANGE, high_pass::HP_CUTOFF_RANGE,
        low_pass::LP_CUTOFF_RANGE,
    },
};

use crate::{
    effects::filters_main::FilterTypes::{BandPass, HighPass, LowPass},
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
        let container_width: u32 = 40;
        let container_height: u32 = 40;
        let diagram_width: i32 = 20;
        let diagram_height: i32 = 20;

        let lp_diagram = FilterDiagram::new(
            FilterTypes::LowPass,
            container_width,
            container_height,
            diagram_width,
            diagram_height,
            200.0,
            self.navigation_location == 1,
            self.filters[0].get_parameters()[0].get_value() == 1.0,
        );
        let bp_diagram = FilterDiagram::new(
            FilterTypes::BandPass,
            container_width,
            container_height,
            diagram_width,
            diagram_height,
            800.0,
            self.navigation_location == 2,
            self.filters[1].get_parameters()[0].get_value() == 1.0,
        );
        let hp_diagram = FilterDiagram::new(
            FilterTypes::HighPass,
            container_width,
            container_height,
            diagram_width,
            diagram_height,
            1000.0,
            self.navigation_location == 3,
            self.filters[2].get_parameters()[0].get_value() == 1.0,
        );

        let filter_options =
            LinearLayout::horizontal(Chain::new(lp_diagram).append(bp_diagram).append(hp_diagram))
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

pub enum FilterTypes {
    LowPass,
    BandPass,
    HighPass,
}
pub struct FilterDiagram {
    filter_type: FilterTypes,
    container_width: u32,
    container_height: u32,
    diagram_width: i32,
    diagram_height: i32,
    cutoff_freq: f32,
    points: [Point; 6],
    selected: bool,
    on: bool,
    position: Point,
}

impl FilterDiagram {
    pub fn new(
        filter_type: FilterTypes,
        container_width: u32,
        container_height: u32,
        diagram_width: i32,
        diagram_height: i32,
        cutoff_freq: f32,
        selected: bool,
        on: bool,
    ) -> Self {
        let points = match filter_type {
            LowPass => get_lp_points(diagram_width, diagram_height, cutoff_freq),
            BandPass => get_bp_points(diagram_width, diagram_height, cutoff_freq),
            HighPass => get_hp_points(diagram_width, diagram_height, cutoff_freq),
        };

        FilterDiagram {
            filter_type,
            container_width,
            container_height,
            diagram_width,
            diagram_height,
            cutoff_freq,
            points,
            selected,
            on,
            position: Point::zero(),
        }
    }

    fn arrange(&self) -> impl View + Drawable<Color = BinaryColor, Output = ()> {
        let container =
            get_diagram_container(self.container_width, self.container_height, self.selected);
        let curve = Polyline::new(&self.points).into_styled(LINE_STYLE);

        let toggle = Text::new(
            match self.on {
                true => "on",
                false => "off",
            },
            Point::zero(),
            SMALL_TEXT_STYLE,
        );

        Chain::new(container)
            .append(curve.align_to(&container, horizontal::Center, vertical::Center))
            .append(toggle.align_to(&container, horizontal::Center, vertical::Top))
            .translate(self.position)
    }
}

impl View for FilterDiagram {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl Drawable for FilterDiagram {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}

type DiagramContainer = embedded_graphics::primitives::Styled<
    Rectangle,
    embedded_graphics::primitives::PrimitiveStyle<BinaryColor>,
>;

fn get_diagram_container(
    container_width: u32,
    container_height: u32,
    selected: bool,
) -> DiagramContainer {
    Rectangle::new(
        Point::zero(),
        Size {
            width: container_width,
            height: container_height,
        },
    )
    .into_styled(match selected {
        true => SELECTED_OPTION_STYLE,
        false => EMPTY_STYLE,
    })
}

// TODO: actually use log scale for x-axis on these diagrams

fn get_lp_points(diagram_width: i32, diagram_height: i32, cutoff_freq: f32) -> [Point; 6] {
    let cutoff_x = ((cutoff_freq / LP_CUTOFF_RANGE.1) * (diagram_width as f32 / 2.0)) as i32;

    [
        Point { x: 0, y: 0 },
        Point { x: 1, y: 0 },
        Point { x: 2, y: 0 },
        Point { x: cutoff_x, y: 0 },
        Point {
            x: cutoff_x + diagram_width / 10,
            y: diagram_height,
        },
        Point {
            x: diagram_width,
            y: diagram_height,
        },
    ]
}

fn get_bp_points(diagram_width: i32, diagram_height: i32, cutoff_freq: f32) -> [Point; 6] {
    let cutoff_x = (diagram_width as f32 / 4.0
        + (cutoff_freq / (BP_CUTOFF_RANGE.1 - BP_CUTOFF_RANGE.0)) * (diagram_width as f32 / 2.0))
        as i32;

    [
        Point {
            x: 0,
            y: diagram_height,
        },
        Point {
            x: cutoff_x - diagram_width / 5,
            y: diagram_height,
        },
        Point {
            x: cutoff_x - diagram_width / 10,
            y: 0,
        },
        Point {
            x: cutoff_x + diagram_width / 10,
            y: 0,
        },
        Point {
            x: cutoff_x + diagram_width / 5,
            y: diagram_height,
        },
        Point {
            x: diagram_width,
            y: diagram_height,
        },
    ]
}

fn get_hp_points(diagram_width: i32, diagram_height: i32, cutoff_freq: f32) -> [Point; 6] {
    let cutoff_x = ((diagram_width as f32 / 2.0)
        + ((cutoff_freq - HP_CUTOFF_RANGE.0) / HP_CUTOFF_RANGE.1 * (diagram_width as f32 / 2.0)))
        as i32;

    [
        Point {
            x: 0,
            y: diagram_height,
        },
        Point {
            x: 1,
            y: diagram_height,
        },
        Point {
            x: 2,
            y: diagram_height,
        },
        Point {
            x: cutoff_x - diagram_width / 10,
            y: diagram_height,
        },
        Point { x: cutoff_x, y: 0 },
        Point {
            x: diagram_width,
            y: 0,
        },
    ]
}
