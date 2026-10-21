use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_4X6, FONT_6X9},
    },
    pixelcolor::BinaryColor,
    primitives::{Circle, Primitive, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_layout::{
    View,
    align::{Align, horizontal, vertical},
    object_chain::Chain,
};

pub const LINE_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

pub const EMPTY_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::new();
pub const SELECTED_OPTION_STYLE: PrimitiveStyle<BinaryColor> =
    PrimitiveStyle::with_stroke(BinaryColor::On, 1);
pub const FILLED_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);

pub const BASE_TEXT_STYLE: MonoTextStyle<'_, BinaryColor> =
    MonoTextStyle::new(&FONT_6X9, BinaryColor::On);
pub const SMALL_TEXT_STYLE: MonoTextStyle<'_, BinaryColor> =
    MonoTextStyle::new(&FONT_4X6, BinaryColor::On);

pub struct ListItem<'b> {
    name: &'b str,
    selected: bool,
    on: bool,
    position: Point,
}

impl<'b> ListItem<'b> {
    pub fn new(name: &'b str, selected: bool, on: bool) -> Self {
        ListItem {
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
                height: 10,
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
            .append(
                text.align_to(&container, horizontal::Left, vertical::Center)
                    .translate(Point { x: 2, y: 0 }),
            )
            .append(toggle.align_to(&container, horizontal::Right, vertical::Center))
            .translate(self.position)
    }
}

impl<'b> View for ListItem<'b> {
    fn translate_impl(&mut self, by: Point) {
        self.position += by;
    }

    fn bounds(&self) -> Rectangle {
        self.arrange().bounds()
    }
}

impl<'b> Drawable for ListItem<'b> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.arrange().draw(target)
    }
}
