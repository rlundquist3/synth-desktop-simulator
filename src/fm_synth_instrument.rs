use std::collections::VecDeque;
use std::f32::consts::PI;
use std::fmt::Debug;
use std::format;
use std::num::NonZero;

use std::sync::{Arc, Mutex, atomic::Ordering};
use std::time::Duration;

use rodio::Source;

use crate::SAMPLE_RATE;
use crate::amp_envelope::AmpEnvelope;
use crate::effects::Effect;
use crate::effects::echo::Echo;
use crate::effects::filters::low_pass;
use crate::effects::filters::{band_pass, high_pass};
use crate::effects::gain::Gain;
use crate::effects::soft_clipper::SoftClipper;
use crate::fm_synth::FMSynth;
use crate::parameter::{
    Parameter,
    ParameterChange::{self, Decrement, Increment},
    UserParameters,
};
use crate::voices::Voices;

const MOD_INDEX_OPTIONS: &[f32] = &[1.0, 2.0, PI, 4.0, 5.0, 2.0 * PI];
const MOD_INDEX_RENDER: &[&str] = &["1", "2", "π", "4", "5", "2π"];

pub const SAMPLE_HISTORY_SIZE: usize = 512;

#[derive(Clone, Copy, Debug)]
pub struct FreqRatio(pub f32, pub f32);

#[derive(Debug)]
pub struct FMSynthInstrument {
    pub voices: Voices<Arc<Mutex<FMSynth>>>,
    headroom_gain: Box<dyn Effect>,
    envelope: AmpEnvelope,
    parameters: Vec<Parameter>,
    pub effects: Vec<Box<dyn Effect>>,
    sample_history: Arc<Mutex<VecDeque<f32>>>,
}

impl FMSynthInstrument {
    pub fn new() -> Self {
        let envelope = AmpEnvelope::new(0.3, 0.2, 0.8, 0.2);
        let signal_source = FMSynth::new(envelope.clone());
        let voices = Voices::new(
            (0..5)
                .map(|_| Arc::new(Mutex::new(signal_source.clone())))
                .collect(),
        );

        let mut effects: Vec<Box<dyn Effect>> = Vec::new();
        effects.push(Box::new(low_pass::new(200.0, 1.0)));
        effects.push(Box::new(high_pass::new(1000.0, 1.0)));
        effects.push(Box::new(band_pass::new(800.0, 1.0)));
        effects.push(Box::new(SoftClipper::new(3.0)));
        effects.push(Box::new(Echo::new(0.0, 0.0)));

        FMSynthInstrument {
            voices,
            parameters: vec![
                Parameter::new("C", 1.0, 1.0, (1.0, 10.0), |v| format!("{:.0}", v)),
                Parameter::new("M", 1.0, 1.0, (1.0, 10.0), |v| format!("{:.0}", v)),
                Parameter::new(
                    "Mod Idx",
                    2.0,
                    1.0,
                    (0.0, (MOD_INDEX_OPTIONS.len() - 1) as f32),
                    |v| format!("{}", MOD_INDEX_RENDER[v as usize]),
                ),
                Parameter::new("LFO Amp", 0.0, 0.1, (0.0, 5.0), |v| format!("{:.1}", v)),
                Parameter::new("LFO Freq", 0.0, 1.0, (0.0, 20.0), |v| format!("{:.0}Hz", v)),
            ],
            headroom_gain: Box::new(Gain::new(-16.0)),
            envelope,
            effects,
            sample_history: Arc::new(Mutex::new(VecDeque::from(vec![0.0; SAMPLE_HISTORY_SIZE]))),
        }
    }

    pub fn get_sample_history(&self) -> Arc<Mutex<VecDeque<f32>>> {
        self.sample_history.clone()
    }
}

impl Clone for FMSynthInstrument {
    fn clone(&self) -> Self {
        FMSynthInstrument {
            voices: self.voices.clone(),
            parameters: self.parameters.clone(),
            headroom_gain: self.headroom_gain.clone_box(),
            envelope: self.envelope.clone(),
            effects: self.effects.iter().map(|e| e.clone_box()).collect(),
            sample_history: self.sample_history.clone(),
        }
    }
}

impl Iterator for FMSynthInstrument {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let raw = self.voices.next()?;
        let headroom_corrected = self.headroom_gain.process(raw);
        let sample = self
            .effects
            .iter_mut()
            .fold(headroom_corrected, |sample, effect| effect.process(sample))
            .clamp(-1.0, 1.0);

        let mut history = self.sample_history.lock().unwrap();
        history.pop_front();
        history.push_back(sample);

        Some(sample)
    }
}

impl Iterator for Voices<Arc<Mutex<FMSynth>>> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.iter_mut().fold(0.0, |acc: f32, v| {
            let mut voice = v.lock().unwrap();

            if voice.on.load(Ordering::Relaxed) {
                acc + voice.next().unwrap_or(0.0)
            } else if !voice.get_release_complete() {
                if !voice.get_releasing() {
                    voice.set_should_release();
                }

                acc + voice.next().unwrap_or(0.0)
            } else {
                acc
            }
        }))
    }
}

impl UserParameters for FMSynthInstrument {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters
            .iter()
            .cloned()
            .chain(self.envelope.get_parameters())
            .collect()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let synth_param_count = self.parameters.len();

        if index >= synth_param_count {
            return self
                .envelope
                .update_parameter(index - synth_param_count, change);
        }

        let param = self.parameters.get(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);

        param.set_value(updated_value);

        match index {
            0 => {
                self.voices.voices.iter().for_each(|voice| {
                    let mut v = voice.lock().unwrap();
                    let existing = v.get_freq_ratio();
                    v.set_freq_ratio(FreqRatio(updated_value, existing.1))
                });
                Some(param.clone())
            }
            1 => {
                self.voices.voices.iter().for_each(|voice| {
                    let mut v = voice.lock().unwrap();
                    let existing = v.get_freq_ratio();
                    v.set_freq_ratio(FreqRatio(existing.0, updated_value))
                });
                Some(param.clone())
            }
            2 => {
                let mod_index = MOD_INDEX_OPTIONS[updated_value as usize];
                self.voices
                    .voices
                    .iter()
                    .for_each(|voice| voice.lock().unwrap().set_mod_index(mod_index));
                Some(param.clone())
            }
            3 => {
                self.voices
                    .voices
                    .iter()
                    .for_each(|voice| voice.lock().unwrap().set_lfo_amp(updated_value));
                Some(param.clone())
            }
            4 => {
                self.voices
                    .voices
                    .iter()
                    .for_each(|voice| voice.lock().unwrap().set_lfo_freq(updated_value));
                Some(param.clone())
            }
            _ => None,
        }
    }
}

impl Source for FMSynthInstrument {
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
