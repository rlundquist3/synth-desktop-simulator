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
pub struct UniversalCombFilter {
    buffer: Vec<f32>,
    write_index: usize,
    parameters: Vec<Parameter>,
}

impl UniversalCombFilter {
    pub fn new(gain: f32, ff: f32, fb: f32, delay_samples: u32) -> Self {
        let max_samples = (MAX_DELAY_SECS * SAMPLE_RATE as f32) as usize;

        let mut parameters: Vec<Parameter> = vec![Parameter::new(
            "Toggle",
            0.0,
            1.0,
            (0.0, 1.0),
            |v| match v {
                1.0 => format!("on"),
                _ => format!("off"),
            },
        )];

        if gain > 0.0 {
            parameters.push(Parameter::new("Gain", gain, 0.1, (0.0, 1.0), |v| {
                format!("{:.1}", v)
            }));
        }

        if ff > 0.0 {
            parameters.push(Parameter::new("FF", ff, 0.1, (0.0, 1.0), |v| {
                format!("{:.1}", v)
            }));
        }

        if fb < 0.0 {
            parameters.push(Parameter::new("FB", fb, 0.1, (-1.0, 0.0), |v| {
                format!("{:.1}", v)
            }));
        }

        if delay_samples > 0 {
            parameters.push(Parameter::new(
                "Delay",
                delay_samples as f32,
                1.0,
                (0.0, SAMPLE_RATE as f32),
                |v| format!("{:.1}s", v / SAMPLE_RATE as f32),
            ));
        }

        UniversalCombFilter {
            buffer: vec![0.0; max_samples],
            write_index: 0,
            parameters,
        }
    }
}

impl Effect for UniversalCombFilter {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(UniversalCombFilter {
            buffer: vec![0.0; self.buffer.len()],
            write_index: 0,
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let on = self.parameters[0].get_value();

        if on != 1.0 {
            return sample;
        }

        let gain = self.parameters[1].get_value();
        let ff = self.parameters[2].get_value();
        let fb = self.parameters[3].get_value();
        let delay_samples = self.parameters[3].get_value() as usize;

        let max = self.buffer.len();
        let read_index = (self.write_index + max - delay_samples.min(max - 1)) % max;
        let delayed = self.buffer[read_index];

        let result = gain * sample + ff * delayed + fb * delayed;
        self.buffer[self.write_index] = result;
        self.write_index = (self.write_index + 1) % max;

        result
    }

    fn get_name(&self) -> String {
        String::from("Universal Comb Filter")
    }
}

impl UserParameters for UniversalCombFilter {
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

pub fn new_ap(gain: f32, delay_samples: u32) -> UniversalCombFilter {
    UniversalCombFilter::new(gain, 1.0, -1.0 * gain, delay_samples)
}

pub fn new_ffcf(gain: f32, ff: f32) -> UniversalCombFilter {
    UniversalCombFilter::new(gain, ff, 0.0, 0)
}

pub fn new_fbcf(gain: f32, fb: f32) -> UniversalCombFilter {
    UniversalCombFilter::new(gain, 0.0, -1.0 * fb, 0)
}

pub fn new_delay_line(delay_samples: u32) -> UniversalCombFilter {
    UniversalCombFilter::new(0.0, 0.0, 0.0, delay_samples)
}
