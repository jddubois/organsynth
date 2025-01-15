#[derive(Copy, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl Waveform {
    pub fn generate_sample(&self, phase: f32, frequency: f32) -> f32 {
        match self {
            Waveform::Sine => Self::generate_sine_sample(phase),
            Waveform::Square => Self::generate_square_sample(phase),
            Waveform::Sawtooth => Self::generate_sawtooth_sample(phase),
            Waveform::Triangle => Self::generate_organ_sample(phase, frequency),
        }
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
        let max_freq = 2000.0; // Highest frequency where the blend reaches mostly triangle
        let log_base = 10.0; // Base of the logarithm, can tweak for smoothness

        // Normalize frequency to a 0-1 range logarithmically
        let normalized_freq = ((frequency / min_freq).log(log_base)
            / (max_freq / min_freq).log(log_base))
        .clamp(0.0, 1.0);

        // Blend factor is the normalized logarithmic value
        let blend_factor = normalized_freq;

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
