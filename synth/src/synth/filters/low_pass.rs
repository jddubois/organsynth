use super::Filter;

pub struct LowPass {
    cutoff: f32,
    last_output: f32,
}

impl LowPass {
    pub fn new(cutoff: f32) -> Self {
        Self {
            cutoff,
            last_output: 0.0,
        }
    }
}

impl Filter for LowPass {
    fn process(&mut self, input: f32) -> f32 {
        self.last_output += self.cutoff * (input - self.last_output);
        self.last_output
    }
}
