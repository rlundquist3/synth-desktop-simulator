use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

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
    gain_param: Arc<AtomicU32>,
    parameters: Vec<EffectParameter>,
}

impl Gain {
    pub fn new(gain_db: f32) -> Self {
        let gain_p = EffectParameter::new("Gain", gain_db, 0.5);
        let gain_param = Arc::clone(&gain_p.value);

        Gain {
            gain_db,
            gain_linear: db_to_linear_gain(gain_db),
            gain_param,
            parameters: vec![gain_p],
        }
    }
}

impl Effect for Gain {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Gain {
            gain_db: self.gain_db,
            gain_linear: self.gain_linear,
            gain_param: Arc::clone(&self.gain_param),
            parameters: self.parameters.clone(),
        })
    }

    fn process(&mut self, sample: f32) -> f32 {
        let current_db = f32::from_bits(self.gain_param.load(Ordering::Relaxed));
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
