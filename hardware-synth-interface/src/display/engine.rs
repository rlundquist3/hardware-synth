use alloc::vec::Vec;
use core::cell::RefCell;
use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_core::{
    chain::Chain,
    engines::ENGINE_COUNT,
    parameter::{Parameter, UserParameters},
};
use synth_gui::{
    engines::{dual::DualLayout, fm::FMLayout, list::EngineListLayout},
    envelope::EnvelopeLayout,
    lfo::LfoLayout,
};

use crate::{
    SharedChain,
    controls::NAVIGATION_LOCATION,
    display::{Display, DisplayError},
};

pub async fn render_engine_list(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();
    let navigation_location = navigation_rx.get().await;

    let (engine_index, engines) = chain.lock(|c: &RefCell<Chain>| {
        let chain = c.borrow();

        (chain.current_engine_index, chain.get_engine_list())
    });

    EngineListLayout::new(engines, engine_index, display_area, navigation_location)
        .draw(display)?;

    Ok(())
}

pub async fn render_engine_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let (engine_index, parameters) = chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        (
            chain.current_engine_index,
            chain.get_engine().get_parameters().to_vec(),
        )
    });

    match engine_index {
        0 => FMLayout::new(&parameters, display_area).draw(display)?,
        1 => DualLayout::new(&parameters, display_area).draw(display)?,
        _ => {}
    };

    Ok(())
}

pub async fn render_engine_lfo(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let parameters = chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        chain.get_engine().get_parameters()[4..].to_vec()
    });

    LfoLayout::new(&parameters, display_area).draw(display)?;

    Ok(())
}

pub async fn render_engine_envelope(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let parameters = chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        chain.get_engine().get_envelope_parameters().to_vec()
    });

    EnvelopeLayout::new(&parameters, display_area).draw(display)?;

    Ok(())
}
