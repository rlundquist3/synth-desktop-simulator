use crate::{
    effects::Effect,
    parameter::{
        Parameter,
        ParameterChange::{self, Decrement, Increment},
        UserParameters,
    },
};

#[derive(Debug)]
pub struct Reverb {
    parameters: Vec<Parameter>,
}

impl Reverb {
    pub fn new() -> Self {
        Reverb {
            parameters: vec![Parameter::new(
                "Toggle",
                0.0,
                1.0,
                (0.0, 1.0),
                |v| match v {
                    1.0 => format!("on"),
                    _ => format!("off"),
                },
            )],
        }
    }
}

impl Effect for Reverb {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Reverb {
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let on = self.parameters[0].get_value();

        if on != 1.0 {
            return sample;
        }

        sample
    }

    fn get_name(&self) -> String {
        String::from("Universal Comb Filter")
    }
}

impl UserParameters for Reverb {
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
