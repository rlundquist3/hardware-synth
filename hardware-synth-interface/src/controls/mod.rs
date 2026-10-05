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
    effects::{EFFECT_COUNT, filters::FILTER_COUNT},
    engines::fm::FMSynth,
    parameter::{
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};
use synth_gui::effects::effects_main::{EFFECT_CHUNK_COUNT, EFFECT_CHUNK_SIZE};

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
    FiltersDetail,
    EffectsMain,
    EffectsDetail,
}
pub static MODE: Watch<CriticalSectionRawMutex, Mode, 2> = Watch::new();
pub static NAVIGATION_LOCATION: Watch<CriticalSectionRawMutex, usize, 2> = Watch::new();

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
            Mode::FiltersDetail => {}
            Mode::EffectsMain => {}
            Mode::EffectsDetail => {}
        }
        DISPLAY_BUFFER.send(2).await;
    }
}

async fn engine_main_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mut mode_tx = MODE.sender();
    match control_event {
        NavigationRight => {
            mode_tx.send(Mode::EngineLFO);
            return;
        }
        _ => {}
    }

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

async fn engine_lfo_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    match control_event {
        NavigationLeft => {
            mode_tx.send(Mode::EngineMain);
            return;
        }
        NavigationRight => {
            mode_tx.send(Mode::EngineEnvelope);
            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((3, Increment)),
        Encoder1Counterclockwise => Some((3, Decrement)),
        Encoder2Clockwise => Some((4, Increment)),
        Encoder2Counterclockwise => Some((4, Decrement)),
        _ => None,
    } {
        chain.lock(|c: &RefCell<Chain>| {
            let mut chain = c.borrow_mut();

            chain.get_engine().update_parameter(param_index, change);
        });
    };
}

async fn engine_envelope_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    match control_event {
        NavigationLeft => {
            mode_tx.send(Mode::EngineLFO);
            return;
        }
        NavigationRight => {
            mode_tx.send(Mode::FiltersMain);
            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((5, Increment)),
        Encoder1Counterclockwise => Some((5, Decrement)),
        Encoder2Clockwise => Some((6, Increment)),
        Encoder2Counterclockwise => Some((6, Decrement)),
        Encoder3Clockwise => Some((7, Increment)),
        Encoder3Counterclockwise => Some((7, Decrement)),
        Encoder4Clockwise => Some((8, Decrement)),
        Encoder4Counterclockwise => Some((8, Increment)),
        _ => None,
    } {
        chain.lock(|c: &RefCell<Chain>| {
            let mut chain = c.borrow_mut();

            chain.get_engine().update_parameter(param_index, change);
        });
    };
}

async fn filters_main_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    let navigation_tx = NAVIGATION_LOCATION.sender();
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();

    let navigation_location = navigation_rx.get().await;
    match navigation_location {
        0 => match control_event {
            NavigationLeft => {
                mode_tx.send(Mode::EngineEnvelope);
                return;
            }
            NavigationRight => {
                mode_tx.send(Mode::EffectsMain);
                return;
            }
            NavigationUp => {
                navigation_tx.send(1);
                return;
            }
            _ => {}
        },
        _ => match control_event {
            NavigationLeft => {
                match navigation_location {
                    1 => {}
                    _ => {
                        navigation_tx.send(navigation_location - 1);
                    }
                };
                return;
            }
            NavigationRight => {
                if navigation_location < FILTER_COUNT {
                    navigation_tx.send(navigation_location + 1);
                }
                return;
            }
            NavigationDown => {
                navigation_tx.send(0);
                return;
            }
            NavigationEnter => {
                mode_tx.send(Mode::FiltersDetail);
                return;
            }
            _ => {}
        },
    }
}

async fn filters_detail_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    let navigation_tx = NAVIGATION_LOCATION.sender();
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();

    let navigation_location = navigation_rx.get().await;
    match navigation_location {
        0 => {}
        _ => match control_event {
            NavigationEnter => {
                navigation_tx.send(0);
                mode_tx.send(Mode::FiltersMain);
                return;
            }
            _ => {}
        },
    }

    match control_event {
        Encoder1Click => {
            chain.lock(|c: &RefCell<Chain>| {
                let mut chain = c.borrow_mut();
                let filter_index = navigation_location - 1;

                chain.get_filters()[filter_index].toggle();
            });
            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((0, Increment)),
        Encoder1Counterclockwise => Some((0, Decrement)),
        Encoder2Clockwise => Some((1, Increment)),
        Encoder2Counterclockwise => Some((1, Decrement)),
        _ => None,
    } {
        chain.lock(|c: &RefCell<Chain>| {
            let mut chain = c.borrow_mut();
            let filter_index = navigation_location - 1;

            chain.get_filters()[filter_index].update_parameter(param_index, change);
        });
    };
}

async fn effects_main_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    let navigation_tx = NAVIGATION_LOCATION.sender();
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();

    let navigation_location = navigation_rx.get().await;
    match navigation_location {
        0 => match control_event {
            NavigationLeft => {
                mode_tx.send(Mode::FiltersMain);
                return;
            }
            NavigationRight => {}
            NavigationUp => {
                navigation_tx.send(1);
                return;
            }
            _ => {}
        },
        _ => match control_event {
            NavigationUp => {
                if navigation_location % EFFECT_CHUNK_SIZE != 1 {
                    navigation_tx.send(navigation_location - 1);
                };
                return;
            }
            NavigationDown => {
                if navigation_location % EFFECT_CHUNK_SIZE != 0
                    && navigation_location < EFFECT_COUNT
                {
                    navigation_tx.send(navigation_location + 1);
                } else {
                    navigation_tx.send(0);
                }
                return;
            }
            NavigationLeft => {
                if navigation_location > EFFECT_CHUNK_SIZE {
                    navigation_tx.send(navigation_location - EFFECT_CHUNK_SIZE);
                }
                return;
            }
            NavigationRight => {
                if navigation_location < EFFECT_CHUNK_COUNT * EFFECT_CHUNK_SIZE {
                    navigation_tx.send(navigation_location + EFFECT_CHUNK_SIZE);
                }
                return;
            }
            NavigationEnter => {
                mode_tx.send(Mode::EffectsDetail);
                return;
            }
            _ => {}
        },
    }
}

async fn effects_detail_handler(chain: &'static SharedChain, control_event: ControlEvent) {
    let mode_tx = MODE.sender();
    let navigation_tx = NAVIGATION_LOCATION.sender();
    let mut navigation_rx = NAVIGATION_LOCATION.receiver().unwrap();

    let navigation_location = navigation_rx.get().await;
    match navigation_location {
        0 => {}
        _ => match control_event {
            NavigationEnter => {
                navigation_tx.send(0);
                mode_tx.send(Mode::EffectsMain);
                return;
            }
            _ => {}
        },
    }

    match control_event {
        Encoder1Click => {
            chain.lock(|c: &RefCell<Chain>| {
                let mut chain = c.borrow_mut();
                let effect_index = navigation_location - 1;

                chain.get_effects()[effect_index].toggle();
            });

            return;
        }
        _ => {}
    }

    if let Some((param_index, change)) = match control_event {
        Encoder1Clockwise => Some((0, Increment)),
        Encoder1Counterclockwise => Some((0, Decrement)),
        Encoder2Clockwise => Some((1, Increment)),
        Encoder2Counterclockwise => Some((1, Decrement)),
        Encoder3Clockwise => Some((2, Increment)),
        Encoder3Counterclockwise => Some((2, Decrement)),
        Encoder4Clockwise => Some((3, Increment)),
        Encoder4Counterclockwise => Some((3, Decrement)),
        _ => None,
    } {
        chain.lock(|c: &RefCell<Chain>| {
            let mut chain = c.borrow_mut();
            let effect_index = navigation_location - 1;

            chain.get_effects()[effect_index].update_parameter(param_index, change);
        });
    };
}
