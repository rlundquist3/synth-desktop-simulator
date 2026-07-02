use crate::{
    effects::{
        Effect, EffectParameter,
        ParameterChange::{self, Decrement, Increment},
    },
    utils::db_to_linear_gain,
};

#[derive(Debug)]
pub struct Gain {
    gain_db: f32,
    gain_linear: f32,
    parameters: Vec<EffectParameter>,
}

impl Gain {
    pub fn new(gain_db: f32) -> Self {
        Gain {
            gain_db,
            gain_linear: db_to_linear_gain(gain_db),
            parameters: vec![EffectParameter::new("Gain", gain_db, 0.5)],
        }
    }
}

impl Effect for Gain {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Gain {
            gain_db: self.gain_db,
            gain_linear: self.gain_linear,
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let current_db = self.parameters[0].get_value();
        if current_db != self.gain_db {
            self.gain_db = current_db;
            self.gain_linear = db_to_linear_gain(current_db);
        }
        self.gain_linear * sample
    }

    fn get_name(&self) -> String {
        String::from("Gain")
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
        param.set_value(param.get_value() + delta);

        Some(param.clone())
    }
}
