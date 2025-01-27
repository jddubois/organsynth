use super::waveform::Waveform;
use crate::config::StopConfig;

#[derive(Copy, Clone, PartialEq)]
pub struct Stop {
    pub waveform: Waveform,
    pub frequency_ratio: f32,
    pub amplitude_ratio: f32,
}

impl Stop {
    pub fn new(config: &StopConfig) -> Self {
        Self {
            waveform: Waveform::parse(&config.waveform),
            frequency_ratio: config.frequency_ratio,
            amplitude_ratio: config.amplitude_ratio,
        }
    }
}
