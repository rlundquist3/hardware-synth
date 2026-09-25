use core::cell::RefCell;

use alloc::format;
use embassy_sync::{
    blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex},
    channel::Channel,
    watch::Watch,
};
use logger::serial_log;
use synth_core::{
    chain::Chain,
    engines::fm::FMSynth,
    parameter::{
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

use crate::{
    SharedChain,
    controls::ControlEvent::{
        Encoder1Click, Encoder1Clockwise, Encoder1Counterclockwise, Encoder2Click,
        Encoder2Clockwise, Encoder2Counterclockwise, Encoder3Click, Encoder3Clockwise,
        Encoder3Counterclockwise, Encoder4Click, Encoder4Clockwise, Encoder4Counterclockwise,
        NavigationDown, NavigationEnter, NavigationLeft, NavigationRight, NavigationUp,
    },
    display::DISPLAY_BUFFER,
};

pub mod encoders;

#[derive(Clone, Debug)]
pub enum Mode {
    EngineMain,
    EngineEnvelope,
    EngineLFO,
    FiltersMain,
    FilterDetail,
    EffectsMain,
    EffectsDetail,
}
pub static MODE: Watch<CriticalSectionRawMutex, Mode, 2> = Watch::new();

#[derive(Debug)]
pub enum ControlEvent {
    NavigationUp,
    NavigationDown,
    NavigationLeft,
    NavigationRight,
    NavigationEnter,
    Encoder1Clockwise,
    Encoder1Counterclockwise,
    Encoder1Click,
    Encoder2Clockwise,
    Encoder2Counterclockwise,
    Encoder2Click,
    Encoder3Clockwise,
    Encoder3Counterclockwise,
    Encoder3Click,
    Encoder4Clockwise,
    Encoder4Counterclockwise,
    Encoder4Click,
}

pub static CONTROL_BUFFER: Channel<CriticalSectionRawMutex, ControlEvent, 16> = Channel::new();

#[embassy_executor::task]
pub async fn control_handler(chain: &'static SharedChain) {
    let mut mode_rx = MODE.receiver().unwrap();
    let mut mode_tx = MODE.sender();
    mode_tx.send(Mode::EngineMain);

    let mut buffer_rx = CONTROL_BUFFER.receiver();

    loop {
        let control_event = buffer_rx.receive().await;

        let mode = mode_rx.get().await;
        match mode {
            Mode::EngineMain => engine_main_handler(chain, control_event).await,
            Mode::EngineEnvelope => {}
            Mode::EngineLFO => {}
            Mode::FiltersMain => {}
            Mode::FilterDetail => {}
            Mode::EffectsMain => {}
            Mode::EffectsDetail => {}
        }
        DISPLAY_BUFFER.send(2).await;
    }
}

async fn engine_main_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    serial_log(&format!("EngineMain: {:?}", control_event));

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((0, Increment)),
        Encoder1Counterclockwise => Some((0, Decrement)),
        Encoder2Clockwise => Some((1, Increment)),
        Encoder2Counterclockwise => Some((1, Decrement)),
        Encoder3Clockwise => Some((2, Increment)),
        Encoder3Counterclockwise => Some((2, Decrement)),
        _ => None,
    } {
        chain.lock(|c: &RefCell<Chain>| {
            let mut chain = c.borrow_mut();

            chain.get_engine().update_parameter(param_index, change);
        });
    }
}
