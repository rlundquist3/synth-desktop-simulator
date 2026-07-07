pub trait Note: Send + Iterator<Item = f32> {
    fn set_freq(&mut self, freq: f32);
}
