use crate::{
    SAMPLE_RATE,
    effects::{
        Effect, EffectParameter,
        ParameterChange::{self, Decrement, Increment},
    },
};

const MAX_DELAY_SECS: f32 = 1.0;

#[derive(Debug)]
pub struct Echo {
    buffer: Vec<f32>,
    write_index: usize,
    parameters: Vec<EffectParameter>,
}

impl Echo {
    pub fn new(delay: u32, decay: f32) -> Self {
        let delay_secs = delay as f32 / SAMPLE_RATE as f32;
        let max_samples = (MAX_DELAY_SECS * SAMPLE_RATE as f32) as usize;

        Echo {
            buffer: vec![0.0; max_samples],
            write_index: 0,
            parameters: vec![
                EffectParameter::new("Delay", delay_secs, 0.1),
                EffectParameter::new("Decay", decay, 0.1),
            ],
        }
    }
}

impl Effect for Echo {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Echo {
            buffer: vec![0.0; self.buffer.len()],
            write_index: 0,
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let delay_samples = (self.parameters[0].get_value() * SAMPLE_RATE as f32) as usize;
        let decay = self.parameters[1].get_value();

        let max = self.buffer.len();
        let read_index = (self.write_index + max - delay_samples.min(max - 1)) % max;
        let delayed = self.buffer[read_index];

        let result = sample + decay * delayed;
        self.buffer[self.write_index] = result;
        self.write_index = (self.write_index + 1) % max;

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
