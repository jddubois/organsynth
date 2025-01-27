pub trait Filter: Send {
    fn process(&mut self, input: f32) -> f32;
}
