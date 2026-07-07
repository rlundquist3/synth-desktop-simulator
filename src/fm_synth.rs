use std::f32::consts::PI;

use crate::{
    note::Note,
    oscillator::{Oscillator, Waveform::Sine},
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

const MOD_INDEX_OPTIONS: &[f32] = &[1.0, 2.0, 3.0, PI, 4.0, 5.0, 2.0 * PI];

#[derive(Clone, Debug)]
pub struct FreqRatio(pub f32, pub f32);

#[derive(Clone, Debug)]
pub struct FMSynth {
    freq_ratio: FreqRatio,
    mod_index: f32,
    carrier_amp: f32,
    carrier_osc: Oscillator,
    mod_osc: Oscillator,
    lfo_amp: f32,
    lfo: Oscillator,
    parameters: Vec<Parameter>,
}

impl FMSynth {
    pub fn new() -> Self {
        let mut lfo = Oscillator::new(Sine);
        lfo.set_freq(0.0);

        FMSynth {
            freq_ratio: FreqRatio(1.0, 1.0),
            mod_index: 3.0,
            carrier_amp: 1.0,
            carrier_osc: Oscillator::new(Sine),
            mod_osc: Oscillator::new(Sine),
            lfo_amp: 0.0,
            lfo,
            parameters: vec![
                Parameter::new("C", 1.0, 1.0, (1.0, 10.0)),
                Parameter::new("M", 1.0, 1.0, (1.0, 10.0)),
                Parameter::new("Mod Index", 3.0, 1.0, (1.0, MOD_INDEX_OPTIONS.len() as f32)),
                Parameter::new("LFO Amp", 0.0, 0.1, (0.0, 5.0)),
                Parameter::new("LFO Freq", 0.0, 1.0, (0.0, 20.0)),
            ],
        }
    }

    pub fn set_fundamental_freq(&mut self, freq: f32) {
        self.carrier_osc.set_freq(self.freq_ratio.0 as f32 * freq);
        self.mod_osc.set_freq(self.freq_ratio.1 as f32 * freq);
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

impl UserParameters for FMSynth {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get(index)?;
        let value = param.get_value();
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        let updated_value = (value + delta).clamp(param.range.0, param.range.1);

        param.set_value(updated_value);

        match index {
            0 => {
                self.freq_ratio.0 = updated_value;
                Some(param.clone())
            }
            1 => {
                self.freq_ratio.1 = updated_value;
                Some(param.clone())
            }
            2 => {
                self.mod_index = MOD_INDEX_OPTIONS[updated_value as usize];
                Some(param.clone())
            }
            3 => {
                self.lfo_amp = updated_value;
                Some(param.clone())
            }
            4 => {
                self.lfo.set_freq(updated_value);
                Some(param.clone())
            }
            _ => None,
        }
    }
}
