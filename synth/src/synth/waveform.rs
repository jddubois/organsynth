#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Trumpet,
}

/// PolyBLEP anti-aliasing correction.
/// `t` is the current phase (0.0 to 1.0), `dt` is frequency/sample_rate.
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        // Near the start of the period
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        // Near the end of the period
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

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

    pub fn generate_sample(&self, phase: f32, frequency: f32, dt: f32) -> f32 {
        match self {
            Waveform::Sine => Self::generate_sine_sample(phase),
            Waveform::Square => Self::generate_square_sample(phase, dt),
            Waveform::Sawtooth => Self::generate_sawtooth_sample(phase, dt),
            Waveform::Triangle => Self::generate_organ_sample(phase, frequency),
            Waveform::Trumpet => Self::generate_trumpet_sample(phase, frequency, dt),
        }
    }

    fn generate_trumpet_sample(phase: f32, frequency: f32, dt: f32) -> f32 {
        // Additive synthesis with organ reed pipe harmonic profile
        const HARMONIC_AMPS: [f32; 16] = [
            1.0, 0.85, 0.72, 0.60, 0.52, 0.45, 0.38, 0.32,
            0.24, 0.18, 0.14, 0.10, 0.07, 0.05, 0.03, 0.02,
        ];

        // Clamp harmonic count by Nyquist to prevent aliasing
        let max_harmonic = (0.5 / dt) as usize;
        let num_harmonics = max_harmonic.min(16);

        // Frequency-dependent brightness adjustment
        let brightness = if frequency < 150.0 {
            1.15 // boost upper harmonics for low notes
        } else if frequency > 500.0 {
            0.8 // reduce upper harmonics for high notes
        } else {
            1.0
        };

        let two_pi = 2.0 * std::f32::consts::PI;
        let mut sample = 0.0;
        let mut amp_sum = 0.0;

        for i in 0..num_harmonics {
            let harmonic = (i + 1) as f32;
            let mut amp = HARMONIC_AMPS[i];

            // Apply brightness scaling to upper harmonics (3rd and above)
            if i >= 2 {
                amp *= brightness;
            }

            sample += amp * (two_pi * harmonic * phase).sin();
            amp_sum += amp;
        }

        if amp_sum > 0.0 {
            sample / amp_sum
        } else {
            0.0
        }
    }

    fn generate_sine_sample(phase: f32) -> f32 {
        (2.0 * std::f32::consts::PI * phase).sin()
    }

    fn generate_square_sample(phase: f32, dt: f32) -> f32 {
        let mut sample = if phase < 0.5 { 1.0 } else { -1.0 };
        sample += poly_blep(phase, dt);
        sample -= poly_blep((phase + 0.5) % 1.0, dt);
        sample
    }

    fn generate_sawtooth_sample(phase: f32, dt: f32) -> f32 {
        let mut sample = 2.0 * phase - 1.0;
        sample -= poly_blep(phase, dt);
        sample
    }

    fn generate_organ_sample(phase: f32, frequency: f32) -> f32 {
        let min_freq = 50.0;
        let max_freq = 250.0;
        let log_base = 10.0;

        let normalized_freq = ((frequency / min_freq).log(log_base)
            / (max_freq / min_freq).log(log_base))
        .clamp(0.0, 1.0);

        let blend_factor = normalized_freq;

        let sine = (phase * 2.0 * std::f32::consts::PI).sin();
        let triangle = if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        };

        (1.0 - blend_factor) * sine + blend_factor * triangle
    }
}
