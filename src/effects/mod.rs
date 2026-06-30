pub mod echo;
pub mod gain;

pub trait Effect: std::fmt::Debug + Send {
    fn process(&mut self, sample: f32) -> f32;
    fn clone_box(&self) -> Box<dyn Effect>;
}
