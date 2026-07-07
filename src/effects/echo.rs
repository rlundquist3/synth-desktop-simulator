use crate::{
    SAMPLE_RATE,
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

const MAX_DELAY_SECS: f32 = 1.0;

#[derive(Debug)]
pub struct Echo {
    buffer: Vec<f32>,
    write_index: usize,
    parameters: Vec<Parameter>,
}

impl Echo {
    pub fn new(delay: u32, amp: f32) -> Self {
        let delay_secs = delay as f32 / SAMPLE_RATE as f32;
        let max_samples = (MAX_DELAY_SECS * SAMPLE_RATE as f32) as usize;

        Echo {
            buffer: vec![0.0; max_samples],
            write_index: 0,
            parameters: vec![
                Parameter::new("Delay", delay_secs, 0.1, (0.0, 1.0)),
                Parameter::new("Amp", amp, 0.1, (0.0, 1.0)),
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
        let amp = self.parameters[1].get_value();

        let max = self.buffer.len();
        let read_index = (self.write_index + max - delay_samples.min(max - 1)) % max;
        let delayed = self.buffer[read_index];

        let result = sample + amp * delayed;
        self.buffer[self.write_index] = result;
        self.write_index = (self.write_index + 1) % max;

        result
    }

    fn get_name(&self) -> String {
        String::from("Echo")
    }
}

impl UserParameters for Echo {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        param.set_value((param.get_value() + delta).clamp(param.range.0, param.range.1));

        Some(param.clone())
    }
}
