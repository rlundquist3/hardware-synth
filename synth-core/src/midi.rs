#[derive(Debug)]
pub struct MidiMessage(pub u8, pub u8, pub u8);

const NO_BEND: u16 = 8192;
const MAX_BEND_UP: u16 = 16383;
const MAX_BEND_DOWN: u16 = 0;

/// converts a midi pitch bend message to a value in the 0-16383 (7-bit unsigned) range
/// (224, 0, 64) => 8192 => no pitch bend
/// (224, 127, 127) => 16383 => max bend up
/// (224, 0, 0) => 0 => max bend down
pub fn get_pitch_bend_value(MidiMessage(_status, lsb, msb): MidiMessage) -> u16 {
    ((msb as u16) << 7) | (lsb as u16)
}

/// given a midi note, midi bend value, and semitone range (+/-),
/// returns a the frequency the note should be bent to, calculated linearly
pub fn get_linear_bent_freq(midi_note: usize, midi_bend: u16, range_semitones: usize) -> f32 {
    let base_freq = MIDI_NOTE_FREQS[midi_note];

    match midi_bend {
        NO_BEND => base_freq,
        _ => {
            let (min, max, bend) = match midi_bend < NO_BEND {
                true => (
                    MIDI_NOTE_FREQS[midi_note - range_semitones],
                    base_freq,
                    midi_bend as f32 / NO_BEND as f32,
                ),
                false => (
                    base_freq,
                    MIDI_NOTE_FREQS[midi_note + range_semitones],
                    (midi_bend - NO_BEND) as f32 / (MAX_BEND_UP - NO_BEND) as f32,
                ),
            };

            min + bend * (max - min)
        }
    }
}

// TODO: split into logic and firmware crates so testing is possible
// #[cfg(test)]
// mod tests {
//     use core::assert_eq;

//     use crate::midi::{MidiMessage, util::get_pitch_bend_value};

//     #[test]
//     fn no_pitch_bend() {
//         assert_eq!(get_pitch_bend_value(MidiMessage(224, 0, 64)), 8192u16);
//     }
//     #[test]
//     fn max_pitch_bend_up() {
//         assert_eq!(get_pitch_bend_value(MidiMessage(224, 127, 127)), 16383u16);
//     }
//     #[test]
//     fn max_pitch_bend_down() {
//         assert_eq!(get_pitch_bend_value(MidiMessage(224, 0, 0)), 0u16);
//     }
// }

/// index = MIDI Note Number, value = freq in Hz
pub const MIDI_NOTE_FREQS: [f32; 128] = [
    8.176, 8.662, 9.177, 9.723, 10.301, 10.913, 11.562, 12.250, 12.978, 13.750, 14.568, 15.434,
    16.352, 17.324, 18.354, 19.445, 20.602, 21.827, 23.125, 24.500, 25.957, 27.500, 29.135, 30.868,
    32.703, 34.648, 36.708, 38.891, 41.203, 43.654, 46.249, 48.999, 51.913, 55.000, 58.270, 61.735,
    65.406, 69.296, 73.416, 77.782, 82.407, 87.307, 92.499, 97.999, 103.826, 110.000, 116.541,
    123.471, 130.813, 138.591, 146.832, 155.563, 164.814, 174.614, 184.997, 195.998, 207.652,
    220.000, 233.082, 246.942, 261.626, 277.183, 293.665, 311.127, 329.628, 349.228, 369.994,
    391.995, 415.305, 440.000, 466.164, 493.883, 523.251, 554.365, 587.330, 622.254, 659.255,
    698.456, 739.989, 783.991, 830.609, 880.000, 932.328, 987.767, 1046.502, 1108.731, 1174.659,
    1244.508, 1318.510, 1396.913, 1479.978, 1567.982, 1661.219, 1760.000, 1864.655, 1975.533,
    2093.005, 2217.461, 2349.318, 2489.016, 2637.020, 2793.826, 2959.955, 3135.963, 3322.438,
    3520.000, 3729.310, 3951.066, 4186.009, 4434.922, 4698.636, 4978.032, 5274.041, 5587.652,
    5919.911, 6271.927, 6644.875, 7040.000, 7458.620, 7902.133, 8372.018, 8869.844, 9397.273,
    9956.063, 10548.080, 11175.300, 11839.820, 12543.850,
];
