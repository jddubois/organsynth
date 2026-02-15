use super::filter::Filter;
use crate::config::ReverbConfig;

/// Comb filter with low-pass damping in the feedback loop
struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    damp1: f32,
    damp2: f32,
    filterstore: f32,
}

impl CombFilter {
    fn new(size: usize, feedback: f32, damping: f32) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
            feedback,
            damp1: damping,
            damp2: 1.0 - damping,
            filterstore: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];

        // Low-pass filter in the feedback path
        self.filterstore = output * self.damp2 + self.filterstore * self.damp1;

        self.buffer[self.index] = input + self.filterstore * self.feedback;

        self.index += 1;
        if self.index >= self.buffer.len() {
            self.index = 0;
        }

        output
    }

}

/// All-pass filter for diffusion
struct AllPassFilter {
    buffer: Vec<f32>,
    index: usize,
}

const ALLPASS_FEEDBACK: f32 = 0.5;

impl AllPassFilter {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            index: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.index];
        let output = -input + buffered;

        self.buffer[self.index] = input + buffered * ALLPASS_FEEDBACK;

        self.index += 1;
        if self.index >= self.buffer.len() {
            self.index = 0;
        }

        output
    }
}

/// Freeverb implementation — Schroeder-Moorer reverb with
/// 8 parallel comb filters and 4 series all-pass filters.
pub struct Freeverb {
    combs: Vec<CombFilter>,
    allpasses: Vec<AllPassFilter>,
    pre_delay: Vec<f32>,
    pre_delay_index: usize,
    wet: f32,
    dry: f32,
}

// Standard Freeverb comb filter delay lengths at 44100 Hz
const COMB_TUNINGS: [usize; 8] = [1557, 1617, 1491, 1422, 1277, 1356, 1188, 1116];
// Standard Freeverb all-pass delay lengths at 44100 Hz
const ALLPASS_TUNINGS: [usize; 4] = [556, 441, 341, 225];
const REFERENCE_RATE: f32 = 44100.0;

impl Freeverb {
    pub fn new(sample_rate: f32, config: &ReverbConfig) -> Self {
        let rate_scale = sample_rate / REFERENCE_RATE;

        let combs = COMB_TUNINGS
            .iter()
            .map(|&size| {
                let scaled_size = ((size as f32) * rate_scale) as usize;
                CombFilter::new(scaled_size, config.room_size, config.damping)
            })
            .collect();

        let allpasses = ALLPASS_TUNINGS
            .iter()
            .map(|&size| {
                let scaled_size = ((size as f32) * rate_scale) as usize;
                AllPassFilter::new(scaled_size)
            })
            .collect();

        let pre_delay_ms = config.pre_delay_ms.unwrap_or(20.0);
        let pre_delay_samples = ((sample_rate * pre_delay_ms) / 1000.0).round() as usize;

        Self {
            combs,
            allpasses,
            pre_delay: vec![0.0; pre_delay_samples.max(1)],
            pre_delay_index: 0,
            wet: config.wet,
            dry: config.dry,
        }
    }
}

impl Filter for Freeverb {
    fn process(&mut self, input: f32) -> f32 {
        // Pre-delay
        let delayed_input = self.pre_delay[self.pre_delay_index];
        self.pre_delay[self.pre_delay_index] = input;
        self.pre_delay_index = (self.pre_delay_index + 1) % self.pre_delay.len();

        // Sum parallel comb filters
        let mut comb_out = 0.0;
        for comb in self.combs.iter_mut() {
            comb_out += comb.process(delayed_input);
        }

        // Series all-pass filters
        let mut output = comb_out;
        for allpass in self.allpasses.iter_mut() {
            output = allpass.process(output);
        }

        // Wet/dry mix
        output * self.wet + input * self.dry
    }
}
