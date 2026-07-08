use crate::{
    fm_synth_instrument::FreqRatio,
    oscillator::{Oscillator, Waveform::Sine},
    voices::Voice,
};
use std::f32::consts::PI;
use std::sync::{Arc, atomic::AtomicBool};

#[derive(Debug)]
pub struct FMSynth {
    freq_ratio: FreqRatio,
    mod_index: f32,
    carrier_amp: f32,
    carrier_osc: Oscillator,
    mod_osc: Oscillator,
    lfo_amp: f32,
    lfo: Oscillator,
    pub on: Arc<AtomicBool>,
    // add envelope property here
}

impl FMSynth {
    pub fn new() -> Self {
        let mut lfo = Oscillator::new(Sine);
        lfo.set_freq(0.0);

        FMSynth {
            freq_ratio: FreqRatio(1.0, 1.0),
            mod_index: PI,
            carrier_amp: 1.0,
            carrier_osc: Oscillator::new(Sine),
            mod_osc: Oscillator::new(Sine),
            lfo_amp: 0.0,
            lfo,
            on: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_fundamental_freq(&mut self, freq: f32) {
        self.carrier_osc.set_freq(self.freq_ratio.0 as f32 * freq);
        self.mod_osc.set_freq(self.freq_ratio.1 as f32 * freq);
    }

    pub fn get_freq_ratio(&self) -> FreqRatio {
        self.freq_ratio
    }

    pub fn set_freq_ratio(&mut self, freq_ratio: FreqRatio) {
        self.freq_ratio = freq_ratio;
    }

    pub fn set_mod_index(&mut self, mod_index: f32) {
        self.mod_index = mod_index;
    }

    pub fn set_lfo_amp(&mut self, amp: f32) {
        self.lfo_amp = amp;
    }

    pub fn set_lfo_freq(&mut self, freq: f32) {
        self.lfo.set_freq(freq);
    }

    // fn get_mod_amp(self) -> f32 {
    //     self.mod_index * self.mod_osc.freq
    // }
}

impl Clone for FMSynth {
    fn clone(&self) -> Self {
        FMSynth {
            freq_ratio: self.freq_ratio,
            mod_index: self.mod_index,
            carrier_amp: self.carrier_amp,
            carrier_osc: self.carrier_osc.clone(),
            mod_osc: self.mod_osc.clone(),
            lfo_amp: self.lfo_amp,
            lfo: self.lfo.clone(),
            on: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Voice for FMSynth {
    fn set_freq(&mut self, freq: f32) {
        self.set_fundamental_freq(freq);
    }
}

impl Iterator for FMSynth {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let lfo_sample = self.lfo.next_sample();

        let c = self
            .carrier_osc
            .next_phase_with_mod(self.lfo_amp * lfo_sample);
        let m = self.mod_osc.next_phase();

        // multiply by envelope value
        Some(self.carrier_amp * (c + self.mod_index * m.sin()).sin())
    }
}
