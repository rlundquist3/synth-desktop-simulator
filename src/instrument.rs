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
use crate::oscillator::Oscillator;
use crate::oscillator::Waveform;
use crate::voices::Voices;

#[derive(Clone, Debug)]
pub struct Voice {
    pub osc: Arc<Mutex<Oscillator>>,
    pub on: Arc<AtomicBool>,
}

impl Iterator for Voices<Voice> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.iter_mut().fold(0.0, |acc: f32, v| {
            if v.on.load(Ordering::Relaxed) {
                acc + v.osc.lock().unwrap().next().unwrap_or(0.0)
            } else {
                acc
            }
        }))
    }
}

#[derive(Debug)]
pub struct Instrument {
    pub voices: Voices<Voice>,
    pub headroom_gain: Box<dyn Effect>,
    pub effects: Vec<Box<dyn Effect>>,
}

impl Clone for Instrument {
    fn clone(&self) -> Self {
        Instrument {
            voices: self.voices.clone(),
            headroom_gain: self.headroom_gain.clone_box(),
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
        }
    }
}

impl Instrument {
    pub fn new(waveform: Waveform) -> Self {
        let voices = Voices::new(
            (0..5)
                .map(|_| Voice {
                    osc: Arc::new(Mutex::new(Oscillator::new(waveform.clone()))),
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

impl Iterator for Instrument {
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

impl Source for Instrument {
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
