use std::num::NonZero;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use rodio::Source;

use crate::SAMPLE_RATE;
use crate::effects::Effect;
use crate::effects::echo::Echo;
use crate::effects::gain::Gain;
use crate::note::Note;
use crate::voices::Voices;

#[derive(Debug)]
pub struct Voice<T> {
    pub note: Arc<Mutex<T>>,
    pub on: Arc<AtomicBool>,
}

impl<T> Clone for Voice<T> {
    fn clone(&self) -> Self {
        Voice {
            note: Arc::clone(&self.note),
            on: Arc::clone(&self.on),
        }
    }
}

impl<T: Note> Iterator for Voices<Voice<T>> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.iter_mut().fold(0.0, |acc: f32, v| {
            if v.on.load(Ordering::Relaxed) {
                acc + v.note.lock().unwrap().next().unwrap_or(0.0)
            } else {
                acc
            }
        }))
    }
}

#[derive(Debug)]
pub struct Instrument<T> {
    pub voices: Voices<Voice<T>>,
    pub headroom_gain: Box<dyn Effect>,
    pub effects: Vec<Box<dyn Effect>>,
}

impl<T> Clone for Instrument<T> {
    fn clone(&self) -> Self {
        Instrument {
            voices: self.voices.clone(),
            headroom_gain: self.headroom_gain.clone_box(),
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
        }
    }
}

impl<T: Note> Instrument<T> {
    pub fn new(signal_source: T) -> Self {
        let voices = Voices::new(
            (0..5)
                .map(|_| Voice {
                    note: Arc::new(Mutex::new(signal_source.clone())),
                    on: Arc::new(AtomicBool::new(false)),
                })
                .collect(),
        );

        let mut effects: Vec<Box<dyn Effect>> = Vec::new();
        effects.push(Box::new(Echo::new(0, 0.0)));

        Instrument {
            voices,
            headroom_gain: Box::new(Gain::new(-16.0)),
            effects,
        }
    }
}

impl<T: Note> Iterator for Instrument<T> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let raw = self.voices.next()?;
        let headroom_corrected = self.headroom_gain.process(raw);

        Some(
            self.effects
                .iter_mut()
                .fold(headroom_corrected, |sample, effect| effect.process(sample))
                .clamp(-1.0, 1.0),
        )
    }
}

impl<T: Note> Source for Instrument<T> {
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(1).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }

    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
