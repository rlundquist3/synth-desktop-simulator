use rodio::Source;
use std::{f32::consts::PI, num::NonZero, time::Duration};

use crate::{
    SAMPLE_RATE,
    oscillator::{Oscillator, Waveform::Sine},
};

#[derive(Debug)]
pub struct FreqRatio(pub f32, pub f32);

#[derive(Debug)]
pub struct FMSynth {
    freq_ratio: FreqRatio,
    mod_index: f32,
    // mod_amp: f32,
    carrier_amp: f32,
    carrier_osc: Oscillator,
    mod_osc: Oscillator,
}

impl FMSynth {
    pub fn new(freq_ratio: FreqRatio, mod_index: f32) -> Self {
        FMSynth {
            freq_ratio,
            mod_index,
            // mod_amp: 1.0,
            carrier_amp: 1.0, // TODO: setter for carrier amp?
            carrier_osc: Oscillator::new(Sine),
            mod_osc: Oscillator::new(Sine),
        }
    }

    pub fn set_fundamental_freq(&mut self, freq: f32) {
        self.carrier_osc.set_freq(self.freq_ratio.0 as f32 * freq);
        self.mod_osc.set_freq(self.freq_ratio.1 as f32 * freq);
    }

    fn get_mod_amp(self) -> f32 {
        self.mod_index * self.mod_osc.freq
    }
}

impl Iterator for FMSynth {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let c = self.carrier_osc.next_phase();
        let m = self.mod_osc.next_phase();

        Some(self.carrier_amp * (c + self.mod_index * m.sin()).sin())
    }
}

impl Source for FMSynth {
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
