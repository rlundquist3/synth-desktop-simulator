use crate::{
    SAMPLE_RATE,
    effects::Effect,
    oscillator::{Oscillator, Waveform::Sine},
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

const MAX_DELAY_SECS: f32 = 15.0 / 1000.0;

#[derive(Debug)]
pub struct Flanger {
    lfo: Oscillator,
    buffer: Vec<f32>,
    write_index: usize,
    parameters: Vec<Parameter>,
}

impl Flanger {
    pub fn new(delay_ms: f32, amp: f32, freq: f32) -> Self {
        let mut lfo = Oscillator::new(Sine);
        lfo.set_freq(freq);
        let max_samples = (MAX_DELAY_SECS * SAMPLE_RATE as f32) as usize;

        Flanger {
            lfo,
            buffer: vec![0.0; max_samples],
            write_index: 0,
            parameters: vec![
                Parameter::new("Toggle", 0.0, 1.0, (0.0, 1.0), |v| match v {
                    1.0 => format!("on"),
                    _ => format!("off"),
                }),
                Parameter::new("Delay", delay_ms, 1.0, (0.0, 15.0), |v| {
                    format!("{:.0}ms", v)
                }),
                Parameter::new("Amp", amp, 0.05, (0.0, 5.0), |v| format!("{:.2}", v)),
                Parameter::new("Freq", freq, 0.1, (0.0, 10.0), |v| format!("{:.1}Hz", v)),
            ],
        }
    }
}

impl Effect for Flanger {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Flanger {
            lfo: self.lfo.clone(),
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

        let raw_delay_samples =
            (self.parameters[1].get_value() / 1000.0 * SAMPLE_RATE as f32) as usize;
        let lfo_amp = self.parameters[2].get_value();
        let lfo_sample: f32 = self.lfo.next_sample();

        let delay_samples_mod = lfo_sample * lfo_amp * raw_delay_samples as f32;
        let delay_samples_float = (raw_delay_samples as f32 + delay_samples_mod)
            .clamp(1.0, self.buffer.len() as f32 - 2.0);

        let mut read_index_float = self.write_index as f32 - delay_samples_float;
        if read_index_float < 0.0 {
            read_index_float += self.buffer.len() as f32;
        }

        // linear interpolation of delayed samples
        let i_0 = read_index_float.floor() as usize;
        let i_1 = (i_0 + 1) % self.buffer.len();
        let f = read_index_float - i_0 as f32;
        let delayed = (1.0 - f) * self.buffer[i_0] + f * self.buffer[i_1];

        let result = 0.5 * sample + 0.5 * delayed;
        self.buffer[self.write_index] = sample;
        self.write_index = (self.write_index + 1) % self.buffer.len();

        result
    }

    fn get_name(&self) -> String {
        String::from("Flanger")
    }
}

impl UserParameters for Flanger {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get(index)?;
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };

        let updated_value = (param.get_value() + delta).clamp(param.range.0, param.range.1);
        param.set_value(updated_value);

        if index == 3 {
            self.lfo.set_freq(updated_value);
        }

        Some(param.clone())
    }
}
