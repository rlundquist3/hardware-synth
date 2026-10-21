use core::cell::RefCell;

use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_core::chain::Chain;
use synth_gui::effects::{effects_detail::EffectsDetailLayout, effects_list::EffectsListLayout};

use crate::{
    SharedChain,
    controls::NAVIGATION_LOCATION,
    display::{Display, DisplayError},
};

pub async fn render_effects_list(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let display_area = display.bounding_box();

    let effects_snapshot = chain.lock(|c: &RefCell<Chain>| c.borrow().get_effects_snapshot());

    EffectsListLayout::new(effects_snapshot, display_area, navigation_location).draw(display)
}

pub async fn render_effects_detail(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let display_area = display.bounding_box();

    let effects_snapshot = chain.lock(|c: &RefCell<Chain>| c.borrow().get_effects_snapshot());

    EffectsDetailLayout::new(effects_snapshot, display_area, navigation_location).draw(display)
}
