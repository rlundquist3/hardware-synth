use crate::parameter::{Parameter, UserParameters};

pub mod fm;

pub trait Engine: Iterator<Item = f32> + UserParameters + Send {
    fn note_on(&mut self, note: u8);
    fn note_off(&mut self, note: u8);
    fn set_pitch_bend(&mut self, bend: u16);
    fn get_envelope_parameters(&self) -> &[Parameter];
    fn get_name(&self) -> &str;
}
