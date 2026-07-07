use rodio::Source;
use std::{f32::consts::PI, num::NonZero, time::Duration};

use crate::SAMPLE_RATE;

#[derive(Clone, Debug)]
pub enum Waveform {
    Sine,
}
pub use self::Waveform::*;

#[derive(Clone, Debug)]
pub struct Oscillator {
    waveform: Waveform,
    pub freq: f32,
    phase: f32,
    phase_delta: f32,
    last_sample: f32,
}

impl Oscillator {
    pub fn new(waveform: Waveform) -> Self {
        Oscillator {
            waveform: waveform,
            freq: 0.0,
            phase: 0.0,
            phase_delta: 0.0,
            last_sample: 0.0,
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.phase_delta = freq * 2.0 * PI / (SAMPLE_RATE as f32);
        self.freq = freq;
    }

    pub fn next_phase(&mut self) -> f32 {
        self.phase += self.phase_delta;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        self.phase
    }

    pub fn next_sample(&mut self) -> f32 {
        self.next_phase();

        self.last_sample = match self.waveform {
            Sine => self.phase.sin(),
        };
        self.last_sample
    }
}

impl Source for Oscillator {
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

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.next_sample())
    }
}
