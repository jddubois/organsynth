use super::{chiff::Chiff, stop::Stop, waveform::Waveform};

pub struct Oscillator {
    phase: f32,
    pub frequency: f32,
    base_frequency: f32,
    sample_rate: f32,
    envelope: Envelope,
    waveform: Waveform,
    amp: f32,
    pub is_released: bool,
    chiff: Chiff,
    /// Pitch LFO state
    pitch_lfo_phase: f32,
    pitch_lfo_rate: f32,    // Hz
    pitch_lfo_depth: f32,   // in cents
    /// Amplitude LFO state
    amp_lfo_phase: f32,
    amp_lfo_rate: f32,      // Hz
    amp_lfo_depth: f32,     // fraction (e.g. 0.04 = ±4%)
}

fn detune(frequency: f32) -> f32 {
    // Random detune ±3 cents per oscillator
    let seed = (frequency * 1000.0) as u32 ^ 0xCAFEBABE;
    let random_val = ((seed.wrapping_mul(2654435761)) % 10000) as f32 / 10000.0; // 0.0 to 1.0
    let cents = (random_val - 0.5) * 6.0; // -3 to +3 cents
    frequency * 2.0_f32.powf(cents / 1200.0)
}

impl Oscillator {
    pub fn from_stop(stop: &Stop, frequency: f32, sample_rate: f32) -> Self {
        Self::new(
            frequency * stop.frequency_ratio,
            sample_rate,
            stop.waveform,
            stop.amplitude_ratio,
            stop.chiff_intensity,
            stop.chiff_duration,
            stop.attack_time,
            stop.release_time,
        )
    }

    pub fn new(
        frequency: f32,
        sample_rate: f32,
        waveform: Waveform,
        amp: f32,
        chiff_intensity: f32,
        chiff_duration: f32,
        attack_time: f32,
        release_time: f32,
    ) -> Self {
        let detuned_freq = detune(frequency);

        // Random initial LFO phases so oscillators don't sync
        let seed = (frequency * 7777.0) as u32;
        let lfo_phase1 = (seed % 1000) as f32 / 1000.0;
        let lfo_phase2 = ((seed.wrapping_mul(31337)) % 1000) as f32 / 1000.0;

        Self {
            phase: 0.0,
            frequency: detuned_freq,
            base_frequency: detuned_freq,
            sample_rate,
            envelope: Envelope::new(sample_rate, attack_time, release_time),
            waveform,
            amp: amp * equal_loudness_multiplier(frequency),
            is_released: false,
            chiff: Chiff::new(frequency, sample_rate, chiff_intensity, chiff_duration, waveform),
            pitch_lfo_phase: lfo_phase1,
            pitch_lfo_rate: 0.15 + (seed % 200) as f32 / 1000.0, // 0.15-0.35 Hz
            pitch_lfo_depth: 2.0 + (seed % 300) as f32 / 100.0,  // 2-5 cents
            amp_lfo_phase: lfo_phase2,
            amp_lfo_rate: 0.2 + ((seed.wrapping_mul(7)) % 200) as f32 / 1000.0, // 0.2-0.4 Hz
            amp_lfo_depth: 0.03 + ((seed.wrapping_mul(13)) % 200) as f32 / 10000.0, // 3-5%
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.advance_phase();
        let dt = self.frequency / self.sample_rate;
        let wave = self.waveform.generate_sample(self.phase, self.frequency, dt);

        // Amplitude LFO modulation
        let amp_mod = 1.0
            + self.amp_lfo_depth
                * (2.0 * std::f32::consts::PI * self.amp_lfo_phase).sin();

        // Mix in chiff transient
        let chiff_sample = self.chiff.next_sample();

        (wave + chiff_sample) * self.amp * amp_mod * self.envelope.next()
    }

    pub fn release(&mut self) {
        self.envelope.trigger_release();
        self.is_released = true;
    }

    pub fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    pub fn matches_stop(&self, stop: &Stop, frequency: f32) -> bool {
        // Use base frequency for comparison (before detune) — approximate match
        let target_freq = frequency * stop.frequency_ratio;
        (self.base_frequency - detune(target_freq)).abs() < 0.01
            && self.waveform == stop.waveform
    }

    fn advance_phase(&mut self) {
        // Pitch LFO modulation
        let pitch_cents = self.pitch_lfo_depth
            * (2.0 * std::f32::consts::PI * self.pitch_lfo_phase).sin();
        self.frequency = self.base_frequency * 2.0_f32.powf(pitch_cents / 1200.0);

        self.phase += self.frequency / self.sample_rate;
        self.phase %= 1.0;

        // Advance LFO phases
        self.pitch_lfo_phase += self.pitch_lfo_rate / self.sample_rate;
        self.pitch_lfo_phase %= 1.0;
        self.amp_lfo_phase += self.amp_lfo_rate / self.sample_rate;
        self.amp_lfo_phase %= 1.0;
    }
}

fn equal_loudness_multiplier(frequency: f32) -> f32 {
    if frequency <= 0.0 {
        return 0.0;
    }

    let f: f32 = frequency;
    let f2 = f * f;

    let f1: f32 = 20.6;
    let f2_const: f32 = 107.7;
    let f3: f32 = 737.9;
    let f4: f32 = 12200.0;
    let ref_freq: f32 = 12194.0;

    let numerator = ref_freq.powi(2) * f.powi(4);
    let denominator = (f2 + f1.powi(2))
        * (f2 + f4.powi(2))
        * ((f2 + f2_const.powi(2)) * (f2 + f3.powi(2))).sqrt();

    let a_db = 20.0 * (numerator / denominator).log10();
    let base_multiplier = 10_f32.powf(-a_db / 20.0);

    const FREQ_THRESHOLD: f32 = 1000.0;
    const FREQ_MAX: f32 = 4186.0;
    const BOOST_MAX_DB: f32 = 4.0;

    let boost_db = if frequency <= FREQ_THRESHOLD {
        0.0
    } else if frequency >= FREQ_MAX {
        BOOST_MAX_DB
    } else {
        let proportion = (frequency - FREQ_THRESHOLD) / (FREQ_MAX - FREQ_THRESHOLD);
        BOOST_MAX_DB * proportion
    };

    let boost_multiplier = 10_f32.powf(boost_db / 20.0);

    (base_multiplier * boost_multiplier * 0.3).min(10.0)
}

// Exponential ADSR envelope (Attack → Sustain → Release)
// Organ pipes don't decay, so we skip the Decay state.
pub struct Envelope {
    pub value: f32,
    attack_coeff: f32,
    release_coeff: f32,
    pub state: EnvelopeState,
}

pub enum EnvelopeState {
    Attack,
    Sustain,
    Release,
    Idle,
}

const EPSILON: f32 = 1e-4;

impl Envelope {
    pub fn new(sample_rate: f32, attack_time: f32, release_time: f32) -> Self {
        // Exponential coefficients: how much to multiply by each sample
        // For attack: we approach 1.0 from a small value
        // For release: we decay toward 0.0
        let attack_coeff = if attack_time > 0.0 {
            (-1.0 / (attack_time * sample_rate)).exp()
        } else {
            0.0 // instant attack
        };
        let release_coeff = if release_time > 0.0 {
            (-1.0 / (release_time * sample_rate)).exp()
        } else {
            0.0 // instant release
        };

        Self {
            value: EPSILON, // Start at a very low value to avoid clicks
            attack_coeff,
            release_coeff,
            state: EnvelopeState::Attack,
        }
    }

    pub fn next(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Attack => {
                // Exponential approach to 1.0:
                // value = 1.0 - (1.0 - value) * coeff
                self.value = 1.0 - (1.0 - self.value) * self.attack_coeff;
                if self.value >= 1.0 - EPSILON {
                    self.value = 1.0;
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Release => {
                // Exponential decay toward 0
                self.value *= self.release_coeff;
                if self.value <= EPSILON {
                    self.value = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
            _ => {}
        }
        self.value
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, EnvelopeState::Idle | EnvelopeState::Release if self.value <= EPSILON)
    }

    pub fn trigger_release(&mut self) {
        self.state = EnvelopeState::Release;
    }
}
