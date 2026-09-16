use alloc::{
    collections::{BTreeMap, VecDeque},
    vec::Vec,
};

pub trait Voice: Clone + Send + Iterator<Item = f32> {
    fn set_freq(&mut self, freq: f32, midi_note: usize);
}

#[derive(Clone, Debug)]
pub struct Voices<T> {
    pub voices: Vec<T>,
    note_voice_map: BTreeMap<u8, usize>,
    free_voices: VecDeque<usize>,
    used_voices: VecDeque<(usize, u8)>,
}

impl<T> Voices<T> {
    pub fn new(voices: Vec<T>) -> Self {
        let voice_count = voices.len();
        let mut free_voices = VecDeque::new();
        for i in 0..voice_count {
            free_voices.push_back(i);
        }

        Voices {
            voices: voices,
            note_voice_map: BTreeMap::new(),
            free_voices: free_voices,
            used_voices: VecDeque::new(),
        }
    }

    pub fn voice_on(&mut self, note: u8) -> &mut T {
        let i = match self.note_voice_map.get(&note) {
            Some(&i) => {
                self.remove_from_used_queue(i);
                i
            }
            None => {
                let i = match self.free_voices.pop_front() {
                    Some(i) => i,
                    None => {
                        let (i, note) = self.used_voices.pop_front().unwrap();
                        self.note_voice_map.remove(&note);
                        i
                    }
                };
                self.note_voice_map.insert(note, i);
                i
            }
        };

        self.used_voices.push_back((i, note));
        &mut self.voices[i]
    }

    pub fn voice_off(&mut self, note: u8) -> Option<&mut T> {
        match self.note_voice_map.remove(&note) {
            Some(i) => {
                self.remove_from_used_queue(i);
                self.free_voices.push_back(i);
                Some(&mut self.voices[i])
            }
            None => None,
        }
    }

    fn remove_from_used_queue(&mut self, index: usize) {
        for i in 0..self.used_voices.len() {
            let (j, _) = self.used_voices[i];
            if j == index {
                self.used_voices.remove(i);
                break;
            }
        }
    }
}
