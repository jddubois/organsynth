use super::waveform::Waveform;
use crate::config::StopConfig;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stop {
    pub waveform: Waveform,
    pub frequency_ratio: f32,
    pub amplitude_ratio: f32,
    pub chiff_intensity: f32,
    pub chiff_duration: f32,
    pub attack_time: f32,
    pub release_time: f32,
}

impl Stop {
    pub fn new(config: &StopConfig) -> Self {
        let waveform = Waveform::parse(&config.waveform);
        // Default chiff/envelope values based on waveform family
        let (default_chiff_intensity, default_chiff_duration, default_attack, default_release) =
            match waveform {
                Waveform::Triangle => (0.3, 0.04, 0.03, 0.1),   // Principal
                Waveform::Sine => (0.1, 0.05, 0.08, 0.15),      // Flute
                Waveform::Trumpet => (0.3, 0.03, 0.02, 0.1),    // Reed
                Waveform::Sawtooth => (0.35, 0.03, 0.01, 0.08), // Reed-like
                Waveform::Square => (0.2, 0.04, 0.02, 0.1),     // Mid
            };

        Self {
            waveform,
            frequency_ratio: config.frequency_ratio,
            amplitude_ratio: config.amplitude_ratio,
            chiff_intensity: config.chiff_intensity.unwrap_or(default_chiff_intensity),
            chiff_duration: config.chiff_duration.unwrap_or(default_chiff_duration),
            attack_time: config.attack_time.unwrap_or(default_attack),
            release_time: config.release_time.unwrap_or(default_release),
        }
    }
}
