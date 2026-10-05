use core::cell::RefCell;

use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_core::chain::Chain;
use synth_gui::effects::{effects_detail::EffectsDetailLayout, effects_main::EffectsMainLayout};

use crate::{
    SharedChain,
    controls::NAVIGATION_LOCATION,
    display::{Display, DisplayError},
};

pub async fn render_effects_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let display_area = display.bounding_box();

    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        EffectsMainLayout::new(chain.get_effects(), display_area, navigation_location).draw(display)
    })
}

pub async fn render_effects_detail(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let display_area = display.bounding_box();

    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        EffectsDetailLayout::new(chain.get_effects(), display_area, navigation_location)
            .draw(display)
    })
}
