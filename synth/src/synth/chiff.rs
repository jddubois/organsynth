use super::waveform::Waveform;

/// Generates breathy pipe attack transient noise.
/// When wind first enters a pipe, there's a brief burst of noise
/// before the pipe "speaks" its steady tone.
pub struct Chiff {
    /// Band-pass filter state for coloring the noise
    bp_y1: f32,
    bp_y2: f32,
    bp_a0: f32,
    bp_a1: f32,
    bp_a2: f32,
    bp_b1: f32,
    bp_b2: f32,
    /// Envelope
    elapsed_samples: u32,
    duration_samples: u32,
    intensity: f32,
    /// Simple PRNG state (xorshift32)
    rng_state: u32,
}

impl Chiff {
    pub fn new(
        frequency: f32,
        sample_rate: f32,
        intensity: f32,
        duration: f32,
        waveform: Waveform,
    ) -> Self {
        // Random variation (±20%) on intensity using a simple seed
        let seed = (frequency * 1000.0) as u32 ^ 0xDEADBEEF;
        let variation = ((seed % 400) as f32 / 1000.0) - 0.2; // -0.2 to +0.2
        let intensity = (intensity * (1.0 + variation)).max(0.0);

        let duration_samples = (duration * sample_rate) as u32;

        // Band-pass filter centered on harmonic of the oscillator frequency
        // Different stop families get different chiff character
        // Scale chiff center frequency with pitch — low pipes have chiff closer to fundamental
        let freq_scale = if frequency < 100.0 {
            // Low notes: center near 1.5× fundamental (subtle, close to pipe resonance)
            1.5
        } else if frequency < 300.0 {
            // Mid range: blend from 1.5× to full multiplier
            1.5 + (frequency - 100.0) / 200.0
        } else {
            // Upper range: use full multiplier
            2.5
        };

        let (center_freq, bandwidth) = match waveform {
            Waveform::Triangle => (frequency * freq_scale, frequency * 1.5),       // Principal: tighter bandwidth
            Waveform::Sine => (frequency * (freq_scale + 0.5), frequency * 1.0),   // Pure sine: airy, narrow
            Waveform::Flute => (frequency * (freq_scale + 0.3), frequency * 1.2), // Flute: breathy but focused
            Waveform::Trumpet | Waveform::Sawtooth => (frequency * freq_scale, frequency * 3.0), // Reed: wider
            Waveform::Square => (frequency * freq_scale, frequency * 2.0),
        };

        // Clamp center frequency well below Nyquist to keep the biquad stable
        let nyquist = sample_rate * 0.45;
        let center_freq = center_freq.min(nyquist);
        let bandwidth = bandwidth.min(nyquist);

        // Compute 2nd-order band-pass filter coefficients
        let omega = 2.0 * std::f32::consts::PI * center_freq / sample_rate;
        let q = center_freq / bandwidth.max(1.0);
        let alpha = omega.sin() / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * omega.cos();
        let a2 = 1.0 - alpha;

        Self {
            bp_y1: 0.0,
            bp_y2: 0.0,
            bp_a0: b0 / a0,
            bp_a1: b1 / a0,
            bp_a2: b2 / a0,
            bp_b1: a1 / a0,
            bp_b2: a2 / a0,
            elapsed_samples: 0,
            duration_samples,
            intensity,
            rng_state: seed,
        }
    }

    /// Returns true if the chiff transient has finished
    pub fn is_finished(&self) -> bool {
        self.elapsed_samples >= self.duration_samples
    }

    /// Generate the next chiff sample
    pub fn next_sample(&mut self) -> f32 {
        if self.is_finished() {
            return 0.0;
        }

        // White noise via xorshift32
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        let noise = (self.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        // Apply band-pass filter
        let filtered = self.bp_a0 * noise + self.bp_a1 * self.bp_y1 + self.bp_a2 * self.bp_y2
            - self.bp_b1 * self.bp_y1
            - self.bp_b2 * self.bp_y2;
        // Shift the biquad state — use the filtered output as y[n]
        self.bp_y2 = self.bp_y1;
        self.bp_y1 = filtered;

        // Fast exponential decay envelope
        let t = self.elapsed_samples as f32 / self.duration_samples as f32;
        let envelope = (-4.0 * t).exp(); // Fast exponential decay

        self.elapsed_samples += 1;

        filtered * envelope * self.intensity
    }
}
