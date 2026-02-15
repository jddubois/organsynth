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
            Waveform::Trumpet => Self::generate_trumpet_sample(phase, dt),
        }
    }

    fn generate_trumpet_sample(phase: f32, dt: f32) -> f32 {
        // Band-limited sawtooth with saturation for brassy tone
        let mut saw_wave = 2.0 * phase - 1.0;
        saw_wave -= poly_blep(phase, dt);
        saw_wave.tanh()
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
