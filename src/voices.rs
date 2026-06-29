use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Voices<T> {
    pub voices: Vec<T>,
    key_voice_map: HashMap<char, usize>,
    free_voices: VecDeque<usize>,
    used_voices: VecDeque<(usize, char)>,
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
            key_voice_map: HashMap::new(),
            free_voices: free_voices,
            used_voices: VecDeque::new(),
        }
    }

    pub fn voice_on(&mut self, key: char) -> &mut T {
        let i = match self.key_voice_map.get(&key) {
            Some(&i) => {
                self.remove_from_used_queue(i);
                i
            }
            None => {
                let i = match self.free_voices.pop_front() {
                    Some(i) => i,
                    None => {
                        let (i, key) = self.used_voices.pop_front().unwrap();
                        self.key_voice_map.remove(&key);
                        i
                    }
                };
                self.key_voice_map.insert(key, i);
                i
            }
        };

        self.used_voices.push_back((i, key));
        &mut self.voices[i]
    }

    pub fn voice_off(&mut self, key: char) -> Option<&mut T> {
        match self.key_voice_map.remove(&key) {
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
