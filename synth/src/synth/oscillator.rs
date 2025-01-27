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
        println!("Oscillator::new({}, {})", frequency, amp);
        Self {
            phase: 0.0, //rand::random(),
            frequency: detune(frequency),
            sample_rate,
            envelope: Envelope::new(sample_rate, frequency),
            waveform,
            amp: amp * iso_equal_loudness(frequency),
            is_released: false,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.advance_phase();
        let wave = self.waveform.generate_sample(self.phase, self.frequency);
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
