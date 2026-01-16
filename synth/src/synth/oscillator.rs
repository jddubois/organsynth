use super::{stop::Stop, waveform::Waveform};

// TODO CLEAN UP THIS FILE!

pub struct Oscillator {
    phase: f32,
    pub frequency: f32,
    sample_rate: f32,
    envelope: Envelope,
    waveform: Waveform,
    amp: f32,
    pub is_released: bool,
    filters: Vec<Box<dyn super::filters::Filter>>,
}

// TODO is this good?
fn detune(frequency: f32) -> f32 {
    // let randfreq: f32 = (rand::random::<f32>() - 0.5) * 2.0;
    // frequency + randfreq
    frequency
}

impl Oscillator {
    pub fn from_stop(stop: &Stop, frequency: f32, sample_rate: f32) -> Self {
        Self::new(
            frequency * stop.frequency_ratio,
            sample_rate,
            stop.waveform,
            stop.amplitude_ratio,
        )
    }

    pub fn new(frequency: f32, sample_rate: f32, waveform: Waveform, amp: f32) -> Self {
        println!(
            "Oscillator::new({}, {}, {})",
            frequency,
            amp,
            waveform.str()
        );
        Self {
            phase: 0.0, //rand::random(),
            frequency: detune(frequency),
            sample_rate,
            envelope: Envelope::new(sample_rate, frequency),
            waveform,
            amp: amp * equal_loudness_multiplier(frequency),
            is_released: false,
            filters: vec![Box::new(super::filters::LowPass::new(0.01))],
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.advance_phase();
        let mut wave = self.waveform.generate_sample(self.phase, self.frequency);

        if self.waveform.str() == "trumpet" {
            // wave = self.filters[0].process(wave);
        }
        wave * self.amp * self.envelope.next()
    }

    pub fn release(&mut self) {
        self.envelope.trigger_release();
        self.is_released = true;
    }

    pub fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    // TODO maybe clean this?
    pub fn matches_stop(&self, stop: &Stop, frequency: f32) -> bool {
        self.frequency == frequency * stop.frequency_ratio
            && self.waveform == stop.waveform
            && self.amp == stop.amplitude_ratio * iso_equal_loudness(frequency)
    }

    fn advance_phase(&mut self) {
        self.phase += self.frequency / self.sample_rate;
        self.phase %= 1.0;
    }
}

fn equal_loudness_multiplier(frequency: f32) -> f32 {
    if frequency <= 0.0 {
        return 0.0;
    }

    // First, compute the inverse A‑weighting multiplier.
    let f: f32 = frequency;
    let f2 = f * f;

    // A‑weighting constants
    let f1: f32 = 20.6;
    let f2_const: f32 = 107.7;
    let f3: f32 = 737.9;
    let f4: f32 = 12200.0;
    let ref_freq: f32 = 12194.0;

    // Numerator: ref_freq^2 * f^4
    let numerator = ref_freq.powi(2) * f.powi(4);

    // Denominator: (f^2 + f1^2) * (f^2 + f4^2) * sqrt((f^2 + f2_const^2)*(f^2 + f3^2))
    let denominator = (f2 + f1.powi(2))
        * (f2 + f4.powi(2))
        * ((f2 + f2_const.powi(2)) * (f2 + f3.powi(2))).sqrt();

    // Compute A‑weighting value in dB.
    let a_db = 20.0 * (numerator / denominator).log10();

    // Inverse weighting multiplier:
    let base_multiplier = 10_f32.powf(-a_db / 20.0);

    // Now add an extra boost for high frequencies.
    // We'll define:
    // - A threshold frequency below which no extra boost is applied.
    // - A maximum frequency corresponding to the top key of an 88‑key keyboard (C8 ~4186 Hz)
    //   where we want to apply the maximum boost.
    // - A maximum boost (in dB) that you can adjust (e.g., 3 dB).
    const FREQ_THRESHOLD: f32 = 1000.0; // Boost starts above 1 kHz.
    const FREQ_MAX: f32 = 4186.0; // Top note for an 88‑key keyboard.
    const BOOST_MAX_DB: f32 = 10.0; // Maximum additional boost in dB.

    // Compute boost in dB based on frequency.
    let boost_db = if frequency <= FREQ_THRESHOLD {
        0.0
    } else if frequency >= FREQ_MAX {
        BOOST_MAX_DB
    } else {
        // Linearly interpolate boost from 0 to BOOST_MAX_DB between FREQ_THRESHOLD and FREQ_MAX.
        let proportion = (frequency - FREQ_THRESHOLD) / (FREQ_MAX - FREQ_THRESHOLD);
        BOOST_MAX_DB * proportion
    };

    // Convert boost from dB to a multiplier.
    let boost_multiplier = 10_f32.powf(boost_db / 20.0);

    // The final multiplier is the product of the inverse A‑weighting multiplier and the boost.
    // We clamp the result to 20.0 to prevent clipping on the lowest notes (since we scale by 0.05 later).
    if base_multiplier > 20.0 {
        println!("base_multiplier: {}", base_multiplier);
    }
    (base_multiplier * boost_multiplier * 0.3).min(10.0)
}

fn equal_loudness_multiplier_old(frequency: f32) -> f32 {
    // Avoid division by zero or invalid input.
    if frequency <= 0.0 {
        return 0.0;
    }

    let f: f32 = frequency;
    let f2: f32 = f * f;

    // Constants based on the A-weighting specification.
    let f1: f32 = 20.6;
    let f2_const: f32 = 107.7;
    let f3: f32 = 737.9;
    let f4: f32 = 12200.0;
    let ref_freq: f32 = 12194.0;

    // Compute numerator: ref^2 * f^4
    let numerator = ref_freq.powi(2) * f.powi(4);

    // Compute denominator:
    // (f^2 + f1^2) * (f^2 + f4^2) * sqrt((f^2 + f2_const^2) * (f^2 + f3^2))
    let denominator = (f2 + f1.powi(2))
        * (f2 + f4.powi(2))
        * ((f2 + f2_const.powi(2)) * (f2 + f3.powi(2))).sqrt();

    // Calculate the A-weighting value in decibels.
    let a_db = 20.0 * (numerator / denominator).log10();

    // To compensate for the frequency-dependent loudness perception, we
    // return an amplitude multiplier that is the inverse of A-weighting:
    // multiplier = 10^(-A(f)/20). At 1 kHz, A(1kHz) = 0 dB, so multiplier = 1.
    let amplitude_multiplier = 10_f32.powf(-a_db / 20.0);

    amplitude_multiplier
}

fn iso_equal_loudness(frequency: f32) -> f32 {
    // Constants for the frequency range
    const MIN_FREQ: f32 = 20.0; // Lowest audible frequency
    const MAX_FREQ: f32 = 12500.0; // Highest audible frequency
    const GAIN_FOR_MIN_FREQ: f32 = 0.01; // Minimum gain for the highest frequencies
    const GAIN_FOR_MAX_FREQ: f32 = 1.0; // Maximum gain for the lowest frequencies

    // Clamp frequency to the audible range
    let clamped_freq = frequency.clamp(MIN_FREQ, MAX_FREQ);

    // Normalize frequency logarithmically
    let log_min = MIN_FREQ.ln();
    let log_max = MAX_FREQ.ln();
    let log_freq = clamped_freq.ln();
    let normalized_freq = (log_freq - log_min) / (log_max - log_min);

    // Apply exponential interpolation
    let x = GAIN_FOR_MAX_FREQ * (GAIN_FOR_MIN_FREQ / GAIN_FOR_MAX_FREQ).powf(normalized_freq);
    // println!("iso_equal_loudness: {} -> {}", frequency, x);
    x
    // 1.0

    // We only handle the range [20, 20000] smoothly.
    // Frequencies outside this range are clamped at the boundary.
    // let f = frequency.clamp(20.0, 20_000.0);

    // // This power-law curve goes from 1.0 at 20 Hz down to 0.1 at 20 kHz.
    // // g(f) = (20 / f)^(1/3).
    // let gain = (20.0 / f).powf(1.0 / 3.0);

    // // gain is guaranteed to be in [0.1, 1.0], so no extra clamp needed.
    // gain
    // 1.0
}

pub struct Envelope {
    pub value: f32,
    attack: f32,
    decay: f32,
    release: f32,
    pub state: EnvelopeState,
    pub sample_rate: f32,
}

enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

const EPSILON: f32 = 1e-6;

impl Envelope {
    pub fn new(sample_rate: f32, frequency: f32) -> Self {
        Self {
            value: 1e-6,  // Start at a very low value to avoid clicks
            attack: 0.05, // 10ms attack
            decay: 0.0,
            release: 0.1, // / (frequency / 400.0), // 100ms release
            state: EnvelopeState::Attack,
            sample_rate,
        }
    }

    pub fn next(&mut self) -> f32 {
        // return 1.0;
        match self.state {
            EnvelopeState::Attack => {
                self.value += 1.0 / (self.attack * self.sample_rate);
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.state = EnvelopeState::Sustain;
                }

                // self.value *= 1.1;  // Exponential decay (0.995 ~ 0.2s)
                // if self.value >= 1.0 {
                //     self.value = 1.0;
                //     self.state = EnvelopeState::Sustain;
                // }
            }
            EnvelopeState::Release => {
                self.value -= 1.0 / (self.release * self.sample_rate);
                if self.value <= 0.0 {
                    self.value = 0.0;
                    // self.state = EnvelopeState::Idle;
                }

                // self.value *= 0.995;  // Exponential decay (0.995 ~ 0.2s)
                // if self.value < 0.001 {
                //     self.value = 0.0;
                //     self.state = EnvelopeState::Idle;
                // }
            }
            EnvelopeState::Decay => {
                self.value -= 1.0 / (self.decay * self.sample_rate);
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.state = EnvelopeState::Sustain;
                    // TODO where to decay to?
                }
            }
            _ => {}
        }
        self.value
    }

    pub fn is_finished(&self) -> bool {
        if let EnvelopeState::Release = self.state {
            self.value <= EPSILON
        } else {
            false
        }
    }

    pub fn trigger_release(&mut self) {
        self.state = EnvelopeState::Release;
    }
}
