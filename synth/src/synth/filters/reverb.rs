use super::filter::Filter;

pub struct SimpleReverb {
    delay_line: Vec<f32>, // Circular buffer for the delayed signal
    delay_index: usize,   // Current position in the delay line
    feedback: f32,        // Feedback amount (0.0 to <1.0 for stability)
    mix: f32,             // Wet/dry mix (0.0 = dry, 1.0 = wet)
}

impl SimpleReverb {
    /// Creates a new `SimpleReverb` with the given parameters.
    pub fn new(sample_rate: f32, delay_ms: f32, feedback: f32, mix: f32) -> Self {
        // Convert delay from milliseconds to samples
        let delay_samples = ((sample_rate * delay_ms) / 1000.0).round() as usize;

        SimpleReverb {
            delay_line: vec![0.0; delay_samples], // Initialize delay buffer
            delay_index: 0,
            feedback: feedback.clamp(0.0, 0.99), // Prevent instability
            mix: mix.clamp(0.0, 1.0),            // Ensure valid mix range
        }
    }
}

impl Filter for SimpleReverb {
    /// Processes a single sample through the reverb.
    fn process(&mut self, input: f32) -> f32 {
        // Get the delayed signal from the buffer
        let delayed_signal = self.delay_line[self.delay_index];

        // Calculate the current output (wet/dry mix)
        let wet_signal = delayed_signal * self.mix;
        let dry_signal = input * (1.0 - self.mix);
        let output = dry_signal + wet_signal;

        // Update the delay line with the current input and feedback
        self.delay_line[self.delay_index] = input + delayed_signal * self.feedback;

        // Increment and wrap the delay index
        self.delay_index = (self.delay_index + 1) % self.delay_line.len();

        output
    }
}
