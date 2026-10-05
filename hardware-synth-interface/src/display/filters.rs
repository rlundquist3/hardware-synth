use core::cell::RefCell;

use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_core::chain::Chain;
use synth_gui::effects::{filters_detail::FiltersDetailLayout, filters_main::FiltersMainLayout};

use crate::{
    SharedChain,
    controls::NAVIGATION_LOCATION,
    display::{Display, DisplayError},
};

pub async fn render_filters_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let display_area = display.bounding_box();

    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        FiltersMainLayout::new(chain.get_filters(), display_area, navigation_location).draw(display)
    })
}

pub async fn render_filters_detail(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let display_area = display.bounding_box();

    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        FiltersDetailLayout::new(chain.get_filters(), display_area, navigation_location)
            .draw(display)
    })
}
