use alloc::vec::Vec;
use alloc::{format, vec};
use core::cell::RefCell;
use core::f32::consts::PI;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;

use crate::effects::gain::Gain;
use crate::midi::notes::MIDI_NOTE_FREQS;
use crate::voices::{VOICES, Voice};
use crate::{
    amp_envelope::AmpEnvelope, effects::Effect, engines::fm::fm_synth_voice::FMSynthVoice,
    parameter::Parameter,
};

pub mod fm_synth_voice;

#[derive(Clone, Copy, Debug)]
pub struct FreqRatio(pub f32, pub f32);

const MOD_INDEX_OPTIONS: &[f32] = &[1.0, 2.0, PI, 4.0, 5.0, 2.0 * PI];
const MOD_INDEX_RENDER: &[&str] = &["1", "2", "π", "4", "5", "2π"];

pub static ENGINE: StaticCell<Mutex<CriticalSectionRawMutex, RefCell<FMSynth>>> = StaticCell::new();

#[embassy_executor::task]
pub async fn voice_state_handler(
    engine: &'static Mutex<CriticalSectionRawMutex, RefCell<FMSynth>>,
) {
    let receiver = VOICES.receiver();

    if let Some(mut s) = receiver {
        loop {
            let states = s.changed().await;

            engine.lock(|e| {
                let mut engine = e.borrow_mut();

                states.iter().enumerate().for_each(|(i, state)| {
                    engine.set_voice(i, *state);
                });
            })
        }
    }
}

#[derive(Debug)]
pub struct FMSynth {
    pub voices: Vec<FMSynthVoice>,
    headroom_gain: Gain,
    envelope: AmpEnvelope,
    parameters: Vec<Parameter>,
    // pub effects: Vec<Box<dyn Effect>>,
}

impl FMSynth {
    pub fn new() -> Self {
        let envelope = AmpEnvelope::new(0.3, 0.2, 0.8, 0.2);
        let signal_source = FMSynthVoice::new(envelope.clone());
        let voices = (0..5).map(|_| signal_source.clone()).collect();

        // let mut effects: Vec<Box<dyn Effect>> = Vec::new();

        FMSynth {
            voices,
            parameters: vec![
                Parameter::new("C", 1.0, 1.0, (1.0, 10.0), |v| format!("{:.0}", v)),
                Parameter::new("M", 1.0, 1.0, (1.0, 10.0), |v| format!("{:.0}", v)),
                Parameter::new(
                    "Mod Idx",
                    2.0,
                    1.0,
                    (0.0, (MOD_INDEX_OPTIONS.len() - 1) as f32),
                    |v| format!("{}", MOD_INDEX_RENDER[v as usize]),
                ),
                Parameter::new("LFO Amp", 0.0, 0.025, (0.0, 5.0), |v| format!("{:.3}", v)),
                Parameter::new("LFO Freq", 0.0, 1.0, (0.0, 8.0), |v| format!("{:.0}Hz", v)),
            ],
            headroom_gain: Gain::new(-16.0),
            envelope,
            // effects,
        }
    }

    pub fn set_voice(&mut self, index: usize, (on, note): (bool, usize)) {
        self.voices[index].set_on(on);
        self.voices[index].set_freq(MIDI_NOTE_FREQS[note]);
    }
}

impl Iterator for FMSynth {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let raw = self.voices.iter_mut().fold(0.0, |acc: f32, voice| {
            if voice.on {
                acc + voice.next().unwrap_or(0.0)
            } else if !voice.get_release_complete() {
                if !voice.get_releasing() {
                    voice.set_should_release();
                }

                acc + voice.next().unwrap_or(0.0)
            } else {
                acc
            }
        });

        let headroom_corrected = self.headroom_gain.process(raw);
        let sample =
            headroom_corrected /* self
                .effects
                .iter_mut()
                .fold(headroom_corrected, |sample, effect| effect.process(sample))*/
                .clamp(-1.0, 1.0);

        Some(sample)
    }
}

// impl UserParameters for FMSynth {
//     fn get_parameters(&self) -> Vec<Parameter> {
//         self.parameters
//             .iter()
//             .cloned()
//             .chain(self.envelope.get_parameters())
//             .collect()
//     }

//     fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
//         let synth_param_count = self.parameters.len();

//         if index >= synth_param_count {
//             return self
//                 .envelope
//                 .update_parameter(index - synth_param_count, change);
//         }

//         let param = self.parameters.get(index)?;
//         let delta = match change {
//             Increment => param.delta,
//             Decrement => -param.delta,
//         };
//         let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);

//         param.set_value(updated_value);

//         match index {
//             0 => {
//                 self.voices.voices.iter().for_each(|voice| {
//                     let mut v = voice.lock().unwrap();
//                     let existing = v.get_freq_ratio();
//                     v.set_freq_ratio(FreqRatio(updated_value, existing.1))
//                 });
//                 Some(param.clone())
//             }
//             1 => {
//                 self.voices.voices.iter().for_each(|voice| {
//                     let mut v = voice.lock().unwrap();
//                     let existing = v.get_freq_ratio();
//                     v.set_freq_ratio(FreqRatio(existing.0, updated_value))
//                 });
//                 Some(param.clone())
//             }
//             2 => {
//                 let mod_index = MOD_INDEX_OPTIONS[updated_value as usize];
//                 self.voices
//                     .voices
//                     .iter()
//                     .for_each(|voice| voice.lock().unwrap().set_mod_index(mod_index));
//                 Some(param.clone())
//             }
//             3 => {
//                 self.voices
//                     .voices
//                     .iter()
//                     .for_each(|voice| voice.lock().unwrap().set_lfo_amp(updated_value));
//                 Some(param.clone())
//             }
//             4 => {
//                 self.voices
//                     .voices
//                     .iter()
//                     .for_each(|voice| voice.lock().unwrap().set_lfo_freq(updated_value));
//                 Some(param.clone())
//             }
//             _ => None,
//         }
//     }
// }
