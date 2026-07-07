use std::f32::consts::PI;

use crate::{
    fm_synth_instrument::FreqRatio,
    note::Note,
    oscillator::{Oscillator, Waveform::Sine},
};

#[derive(Clone, Debug)]
pub struct FMSynth {
    freq_ratio: FreqRatio,
    mod_index: f32,
    carrier_amp: f32,
    carrier_osc: Oscillator,
    mod_osc: Oscillator,
    lfo_amp: f32,
    lfo: Oscillator,
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

impl Note for FMSynth {
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

        Some(self.carrier_amp * (c + self.mod_index * m.sin()).sin())
    }
}
