use crate::{
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
    utils::db_to_linear_gain,
};

#[derive(Debug)]
pub struct SoftClipper {
    gain_db: f32,
    gain_linear: f32,
    parameters: Vec<Parameter>,
}

impl SoftClipper {
    pub fn new(gain_db: f32) -> Self {
        SoftClipper {
            gain_db,
            gain_linear: db_to_linear_gain(gain_db),
            parameters: vec![
                Parameter::new("Toggle", 0.0, 1.0, (0.0, 1.0), |v| match v {
                    1.0 => format!("on"),
                    _ => format!("off"),
                }),
                Parameter::new("Gain", gain_db, 0.5, (0.0, 16.0), |v| format!("{:.1}dB", v)),
            ],
        }
    }
}

impl Effect for SoftClipper {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(SoftClipper {
            gain_db: self.gain_db,
            gain_linear: self.gain_linear,
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let on = self.parameters[0].get_value();

        if on != 1.0 {
            return sample;
        }

        let current_db = self.parameters[0].get_value();
        if current_db != self.gain_db {
            self.gain_db = current_db;
            self.gain_linear = db_to_linear_gain(current_db);
        }

        (self.gain_linear * sample).tanh() / self.gain_linear.tanh()
    }

    fn get_name(&self) -> String {
        String::from("Soft Clipper")
    }
}

impl UserParameters for SoftClipper {
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
