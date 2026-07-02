pub mod echo;
pub mod gain;

use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

#[derive(Debug, Clone)]
pub struct EffectParameter {
    pub name: &'static str,
    pub value: Arc<AtomicU32>,
    pub delta: f32,
}

impl EffectParameter {
    pub fn new(name: &'static str, initial: f32, delta: f32) -> Self {
        EffectParameter {
            name,
            value: Arc::new(AtomicU32::new(initial.to_bits())),
            delta,
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

pub trait Effect: std::fmt::Debug + Send {
    fn process(&mut self, sample: f32) -> f32;
    fn clone_box(&self) -> Box<dyn Effect>;
    fn get_name(&self) -> String;
    fn get_parameters(&self) -> Vec<EffectParameter>;
    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<EffectParameter>;
}
