use std::sync::{Arc, atomic::AtomicBool};

use rodio::Source;

use crate::oscillator::Oscillator;
use crate::oscillator::Waveform;
use crate::{filter::Filter, voices::Voices};

/**
 * - voices, all of same type
 * - set of filters
 *   - chain calls to filters
 *   - each defined as a separate class
 *     - whose Iterator.next modifies the signal accordingly
 *     - has functions to tweak parameters
 * - ui adds this (or each of its voices) to mixer
 * - ui calls functions to add filters/modifiers to the chain and tweak their parameters
 */

#[derive(Debug)]
pub struct Voice {
    pub osc: Oscillator,
    pub on: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct Instrument {
    pub voices: Voices<Voice>,
    // filters: Vec<Filter<Source>>,
}

impl Instrument {
    pub fn new(waveform: Waveform) -> Self {
        let mut voices = Vec::new();
        for _ in 0..5 {
            voices.push(Voice {
                osc: Oscillator::new(waveform.clone()),
                on: Arc::new(AtomicBool::new(false)),
            });
        }

        Instrument {
            voices: Voices::new(voices),
            // filters: Vec::new(),
        }
    }
}

impl Iterator for Instrument {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.voices.voices.iter_mut().fold(0.0, |acc: f32, v| {
            acc + match v.osc.next() {
                Some(v) => v,
                None => 0.0,
            }
        }))
    }
}
