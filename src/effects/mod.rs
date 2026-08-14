pub mod echo;
pub mod filters;
pub mod gain;
pub mod reverb;
pub mod soft_clipper;
pub mod universal_comb_filter;

use crate::parameter::UserParameters;

pub trait Effect: std::fmt::Debug + Send + UserParameters {
    fn process(&mut self, sample: f32) -> f32;
    fn clone_box(&self) -> Box<dyn Effect>;
    fn get_name(&self) -> String;
}
