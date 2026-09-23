use alloc::format;
use core::cell::RefCell;
use daisy_embassy::hal::{exti::ExtiInput, mode::Async};
use embassy_sync::{
    blocking_mutex::{Mutex as BlockingMutex, raw::CriticalSectionRawMutex},
    signal::Signal,
};
use embassy_time::{Duration, Instant, Timer};
use logger::serial_log;
use synth_core::engines::fm::FMSynth;

use crate::controls::{
    CONTROL_BUFFER, ControlEvent, MODE,
    Mode::{
        EffectsDetail, EffectsMain, EngineEnvelope, EngineLFO, EngineMain, FilterDetail,
        FiltersMain,
    },
    engine_main_handler,
};

// static ENCODER1: Signal<CriticalSectionRawMutex, EncoderEvent> = Signal::new();

enum EncoderEvent {
    Clockwise,
    Counterclockwise,
    Click,
}

const DELAY: Duration = Duration::from_millis(250);

#[embassy_executor::task]
pub async fn encoder_handler(
    encoder_index: usize,
    mut clk: ExtiInput<'static, Async>,
    dt: ExtiInput<'static, Async>,
) {
    let mut start = Instant::now();
    let mut last_clk_state = clk.get_level();
    let mut change = 0;

    loop {
        clk.wait_for_any_edge().await;
        let clk_state = clk.get_level();

        if last_clk_state != clk_state {
            let dt_state = dt.get_level();

            if let Some(control_event) = match clk_state == dt_state {
                true => {
                    change -= 1;
                    match encoder_index {
                        1 => Some(ControlEvent::Encoder1Counterclockwise),
                        2 => Some(ControlEvent::Encoder2Counterclockwise),
                        3 => Some(ControlEvent::Encoder3Counterclockwise),
                        4 => Some(ControlEvent::Encoder4Counterclockwise),
                        _ => None,
                    }
                }
                false => {
                    change += 1;
                    match encoder_index {
                        1 => Some(ControlEvent::Encoder1Clockwise),
                        2 => Some(ControlEvent::Encoder2Clockwise),
                        3 => Some(ControlEvent::Encoder3Clockwise),
                        4 => Some(ControlEvent::Encoder4Clockwise),
                        _ => None,
                    }
                }
            } {
                if start.elapsed() >= DELAY {
                    // serial_log(&format!("Encoder {:?} {:?}", control_event, change));
                    CONTROL_BUFFER.send(control_event).await;
                    change = 0;
                    start = Instant::now();
                }
            }

            // TODO: figure out how to signal this better (likely different struct than Signal) and maybe debounce
        }

        last_clk_state = clk_state;
    }
}

#[embassy_executor::task]
pub async fn encoder_click_handler(encoder_index: usize, mut sw: ExtiInput<'static, Async>) {
    loop {
        sw.wait_for_low().await;

        // serial_log(&format!("Encoder {:?} clicked", encoder_index));

        if let Some(control_event) = match encoder_index {
            1 => Some(ControlEvent::Encoder1Click),
            2 => Some(ControlEvent::Encoder2Click),
            3 => Some(ControlEvent::Encoder3Click),
            4 => Some(ControlEvent::Encoder4Click),
            _ => None,
        } {
            CONTROL_BUFFER.send(control_event).await;
        }

        Timer::after_millis(250).await;
    }
}

// #[embassy_executor::task]
// pub async fn encoder_1_receiver_test(
//     engine: &'static BlockingMutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
// ) {
//     let mut mode_rx = MODE.receiver().unwrap();

//     loop {
//         let value = ENCODER1.wait().await;
//         serial_log(&format!("Received {:?}", value));

//         let control_event = match value {
//             EncoderEvent::Clockwise => ControlEvent::Encoder1Clockwise,
//             EncoderEvent::Counterclockwise => ControlEvent::Encoder1Counterclockwise,
//             EncoderEvent::Click => ControlEvent::Encoder1Click,
//         };

//         let mode = mode_rx.get().await;
//         match mode {
//             EngineMain => engine_main_handler(engine, control_event).await,
//             EngineEnvelope => {}
//             EngineLFO => {}
//             FiltersMain => {}
//             FilterDetail => {}
//             EffectsMain => {}
//             EffectsDetail => {}
//         }
//     }
// }
