use crate::{effects::Effect, utils::db_to_linear_gain};

#[derive(Debug)]
pub struct Gain {
    gain_db: f32,
    gain_linear: f32,
}

impl Gain {
    pub fn new(gain_db: f32) -> Self {
        Gain {
            gain_db,
            gain_linear: db_to_linear_gain(gain_db),
        }
    }
}

impl Effect for Gain {
    fn clone_box(&self) -> Box<dyn Effect> {
        Box::new(Gain::new(self.gain_db))
    }

    fn process(&mut self, sample: f32) -> f32 {
        self.gain_linear * sample
    }
}
