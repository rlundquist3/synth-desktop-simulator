use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::{
    SAMPLE_RATE,
    effects::{
        Effect, EffectParameter,
        ParameterChange::{self, Decrement, Increment},
    },
};

const MAX_DELAY_SECS: f32 = 2.0;

#[derive(Debug)]
pub struct Echo {
    delay_param: Arc<AtomicU32>,
    decay_param: Arc<AtomicU32>,
    buffer: Vec<f32>,
    write_pos: usize,
    parameters: Vec<EffectParameter>,
}

impl Echo {
    pub fn new(delay: u32, decay: f32) -> Self {
        let delay_secs = delay as f32 / SAMPLE_RATE as f32;
        let delay_p = EffectParameter::new("Delay", delay_secs, 0.1);
        let decay_p = EffectParameter::new("Decay", decay, 0.1);

        let delay_param = Arc::clone(&delay_p.value);
        let decay_param = Arc::clone(&decay_p.value);

        let max_samples = (MAX_DELAY_SECS * SAMPLE_RATE as f32) as usize;

        Echo {
            delay_param,
            decay_param,
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            parameters: vec![delay_p, decay_p],
        }
    }
}

impl Effect for Echo {
    fn clone_box(&self) -> Box<dyn Effect> {
        let max_samples = self.buffer.len();
        Box::new(Echo {
            delay_param: Arc::clone(&self.delay_param),
            decay_param: Arc::clone(&self.decay_param),
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let delay_secs = f32::from_bits(self.delay_param.load(Ordering::Relaxed));
        let delay_samples = (delay_secs * SAMPLE_RATE as f32) as usize;
        let decay = f32::from_bits(self.decay_param.load(Ordering::Relaxed));

        let max = self.buffer.len();
        let read_pos = (self.write_pos + max - delay_samples.min(max - 1)) % max;
        let delayed = self.buffer[read_pos];

        let result = sample + decay * delayed;
        self.buffer[self.write_pos] = result;
        self.write_pos = (self.write_pos + 1) % max;

        result
    }

    fn get_name(&self) -> String {
        String::from("Echo")
    }

    fn get_parameters(&self) -> Vec<EffectParameter> {
        self.parameters.clone()
    }

    fn update_parameter(
        &mut self,
        index: usize,
        change: ParameterChange,
    ) -> Option<EffectParameter> {
        let param = self.parameters.get(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        param.set_value((param.get_value() + delta).clamp(0.0, 1.0));
        Some(param.clone())
    }
}
