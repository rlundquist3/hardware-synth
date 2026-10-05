use alloc::{format, vec::Vec};
use core::cell::RefCell;
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex};
use embedded_graphics::{Drawable, geometry::Dimensions};
use synth_core::{
    chain::Chain,
    engines::fm::FMSynth,
    parameter::{Parameter, UserParameters},
};
use synth_gui::{engines::fm::EngineMainLayout, envelope::EnvelopeLayout, lfo::LfoLayout};

use crate::{
    SharedChain,
    display::{Display, DisplayError},
};

pub async fn render_engine_main(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let mut parameters: Vec<Parameter> = Vec::new();
    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        parameters = chain.get_engine().get_parameters();
    });

    EngineMainLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}

pub async fn render_engine_lfo(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let mut parameters: Vec<Parameter> = Vec::new();
    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        parameters = chain.get_engine().get_parameters().clone().split_off(3)
    });

    LfoLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}

pub async fn render_engine_envelope(
    display: &mut Display,
    chain: &'static SharedChain,
) -> Result<(), DisplayError> {
    let display_area = display.bounding_box();

    let mut parameters: Vec<Parameter> = Vec::new();
    chain.lock(|c: &RefCell<Chain>| {
        let mut chain = c.borrow_mut();

        parameters = chain.get_engine().get_envelope_parameters();
    });

    EnvelopeLayout::new(parameters, display_area).draw(display)?;

    Ok(())
}
