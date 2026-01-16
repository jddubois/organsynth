#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Trumpet,
}

fn sigmoid_saturation(x: f32, drive: f32) -> f32 {
    x / (1.0 + drive * x.abs()) // Adjust `drive` to control saturation intensity
}

fn soft_clip(x: f32) -> f32 {
    if x > 1.0 {
        1.0
    } else if x < -1.0 {
        -1.0
    } else {
        x - (x * x * x / 3.0) // A polynomial soft-clipping function
    }
}

fn soft_wavefold(x: f32) -> f32 {
    (x.abs() + 0.5) % 1.0 * x.signum() // Folds the waveform gently
}

// TODO clean up this file!
impl Waveform {
    pub fn str(&self) -> &str {
        match self {
            Waveform::Sine => "sine",
            Waveform::Square => "square",
            Waveform::Sawtooth => "sawtooth",
            Waveform::Triangle => "triangle",
            Waveform::Trumpet => "trumpet",
        }
    }
    pub fn parse(waveform: &str) -> Self {
        match waveform {
            "sine" => Waveform::Sine,
            "square" => Waveform::Square,
            "sawtooth" => Waveform::Sawtooth,
            "triangle" => Waveform::Triangle,
            "trumpet" => Waveform::Trumpet,
            _ => Waveform::Sine,
        }
    }

    pub fn generate_sample(&self, phase: f32, frequency: f32) -> f32 {
        match self {
            Waveform::Sine => Self::generate_sine_sample(phase),
            Waveform::Square => Self::generate_square_sample(phase),
            Waveform::Sawtooth => Self::generate_sawtooth_sample(phase),
            Waveform::Triangle => Self::generate_organ_sample(phase, frequency),
            Waveform::Trumpet => Self::generate_trumpet_sample(phase, frequency),
        }
    }

    fn generate_trumpet_sample(phase: f32, freq: f32) -> f32 {
        // 1. Generate a Sawtooth Wave
        let saw_wave = 2.0 * phase - 1.0;

        // 2. Add a Phase-Dependent Pulse Wave (Approximating PWM)
        let pulse_width = 0.1; // + 0.1 * (freq * 0.01).sin(); // Simulated slow modulation
        let pulse_wave = if phase < pulse_width { 1.0 } else { -1.0 };

        // 3. Mix the Saw and Pulse Waves for a Richer Harmonic Structure
        let mix = 0.99 * saw_wave + 0.01 * pulse_wave;

        // 4. Simulate Formant Filtering (Resonance at ~1.5 kHz)
        let formant = mix * (1.0 - 0.7 * (phase * std::f32::consts::PI * 2.0).sin());

        // 5. Apply Soft Saturation for a Brassy Tone
        (saw_wave).tanh()
        // sigmoid_saturation(saw_wave, 1.0)
        // saw_wave.signum() * saw_wave.abs().sqrt()
    }

    fn generate_sine_sample(phase: f32) -> f32 {
        (2.0 * std::f32::consts::PI * phase).sin()
    }

    fn generate_square_sample(phase: f32) -> f32 {
        if phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }

    fn generate_sawtooth_sample(phase: f32) -> f32 {
        2.0 * (phase - 0.5)
    }

    fn generate_organ_sample(phase: f32, frequency: f32) -> f32 {
        // Parameters for the logarithmic curve
        let min_freq = 20.0; // Lowest frequency (e.g., 20 Hz)
        let max_freq = 100.0; // Highest frequency where the blend reaches mostly triangle
        let log_base = 10.0; // Base of the logarithm, can tweak for smoothness

        // Normalize frequency to a 0-1 range logarithmically
        let normalized_freq = ((frequency / min_freq).log(log_base)
            / (max_freq / min_freq).log(log_base))
        .clamp(0.0, 1.0);

        // Blend factor is the normalized logarithmic value
        let blend_factor = normalized_freq;

        // println!("Freq: {} Blend factor: {}", frequency, blend_factor);

        // 2. Generate the two waveforms:
        let sine = (phase * 2.0 * std::f32::consts::PI).sin();
        let triangle = if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        };

        // 3. Crossfade between sine and triangle
        //    blend_factor = 0 => all sine
        //    blend_factor = 1 => all triangle
        (1.0 - blend_factor) * sine + blend_factor * triangle
    }

    fn generate_triangle_sample(phase: f32) -> f32 {
        if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        }
    }
}
