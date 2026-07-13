use std::f32::consts::PI;

use crate::{
    SAMPLE_RATE,
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

#[derive(Debug)]
pub struct LowPass {
    parameters: Vec<Parameter>,
    a: (f32, f32, f32),
    b: (f32, f32, f32),
    s_1: f32,
    s_2: f32,
}

impl LowPass {
    pub fn new(cutoff_freq: f32, q: f32) -> Self {
        let (a, b) = get_normalized_coefficients(cutoff_freq, q);

        LowPass {
            parameters: vec![
                Parameter::new("Toggle", 0.0, 1.0, (0.0, 1.0), |v| match v {
                    1.0 => format!("on"),
                    _ => format!("off"),
                }),
                Parameter::new("Cutoff Freq", cutoff_freq, 10.0, (20.0, 600.0), |v| {
                    format!("{:.0}Hz", v)
                }),
                Parameter::new("Q", q, 0.1, (0.1, 30.0), |v| format!("{:.1}", v)),
            ],
            a,
            b,
            s_1: 0.0,
            s_2: 0.0,
        }
    }

    fn recalculate_coefficients(&mut self) {
        let cutoff_freq = self.parameters[1].get_value();
        let q = self.parameters[2].get_value();

        let (a, b) = get_normalized_coefficients(cutoff_freq, q);

        self.a = a;
        self.b = b;
    }
}

impl Effect for LowPass {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(LowPass {
            parameters: self.parameters.clone(),
            a: self.a.clone(),
            b: self.b.clone(),
            s_1: self.s_1.clone(),
            s_2: self.s_2.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let on = self.parameters[0].get_value();

        if on != 1.0 {
            return sample;
        }

        let result = self.b.0 * sample + self.s_1;
        self.s_1 = self.b.1 * sample + self.s_2 - self.a.1 * result;
        self.s_2 = self.b.2 * sample - self.a.2 * result;

        result
    }

    fn get_name(&self) -> String {
        String::from("Low Pass")
    }
}

impl UserParameters for LowPass {
    fn get_parameters(&self) -> Vec<Parameter> {
        self.parameters.clone()
    }

    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter> {
        let param = self.parameters.get(index)?.clone();
        let delta = match change {
            Increment => param.delta,
            Decrement => -param.delta,
        };
        param.set_value((param.get_value() + delta).clamp(param.range.0, param.range.1));

        if index != 0 {
            self.recalculate_coefficients();
        }

        Some(param.clone())
    }
}

fn get_normalized_coefficients(cutoff_freq: f32, q: f32) -> ((f32, f32, f32), (f32, f32, f32)) {
    let k: f32 = (PI * cutoff_freq / SAMPLE_RATE as f32).tan();
    let a_0 = 1.0 + k / q + k.powf(2.0);

    let b = (
        k.powf(2.0) / a_0,
        2.0 * k.powf(2.0) / a_0,
        k.powf(2.0) / a_0,
    );
    let a = (
        1.0,
        (2.0 * k.powf(2.0) - 2.0) / a_0,
        (1.0 - k / q + k.powf(2.0)) / a_0,
    );

    (a, b)
}
