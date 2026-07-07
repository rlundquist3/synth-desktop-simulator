use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: &'static str,
    pub value: Arc<AtomicU32>, // value is Arc<AtomicU32> so it can be used safely across both the audio and UI threads
    pub delta: f32,
    pub range: (f32, f32),
}

impl Parameter {
    pub fn new(name: &'static str, initial: f32, delta: f32, range: (f32, f32)) -> Self {
        Parameter {
            name,
            value: Arc::new(AtomicU32::new(initial.to_bits())),
            delta,
            range,
        }
    }

    pub fn get_value(&self) -> f32 {
        f32::from_bits(self.value.load(Ordering::Relaxed))
    }

    pub fn set_value(&self, v: f32) {
        self.value.store(v.to_bits(), Ordering::Relaxed);
    }
}

pub enum ParameterChange {
    Increment,
    Decrement,
}

pub trait UserParameters: std::fmt::Debug + Send {
    fn get_parameters(&self) -> Vec<Parameter>;
    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter>;
}
