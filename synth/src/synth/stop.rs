use super::waveform::Waveform;
use crate::config::{BreakingStopConfig, StopConfig};

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
        let defaults = waveform_defaults(waveform);

        Self {
            waveform,
            frequency_ratio: config.frequency_ratio,
            amplitude_ratio: config.amplitude_ratio,
            chiff_intensity: config.chiff_intensity.unwrap_or(defaults.0),
            chiff_duration: config.chiff_duration.unwrap_or(defaults.1),
            attack_time: config.attack_time.unwrap_or(defaults.2),
            release_time: config.release_time.unwrap_or(defaults.3),
        }
    }
}

fn waveform_defaults(waveform: Waveform) -> (f32, f32, f32, f32) {
    match waveform {
        Waveform::Triangle => (0.3, 0.04, 0.03, 0.1),
        Waveform::Sine => (0.1, 0.05, 0.08, 0.15),
        Waveform::Flute => (0.08, 0.06, 0.06, 0.18),
        Waveform::Trumpet => (0.3, 0.03, 0.015, 0.15),
        Waveform::Sawtooth => (0.35, 0.03, 0.01, 0.08),
        Waveform::Square => (0.2, 0.04, 0.02, 0.1),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakPoint {
    pub note: u8,
    pub frequency_ratio: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakingStop {
    pub waveform: Waveform,
    pub amplitude_ratio: f32,
    pub chiff_intensity: f32,
    pub chiff_duration: f32,
    pub attack_time: f32,
    pub release_time: f32,
    pub break_points: Vec<BreakPoint>,
}

impl BreakingStop {
    pub fn new(config: &BreakingStopConfig) -> Self {
        let waveform = Waveform::parse(&config.waveform);
        let defaults = waveform_defaults(waveform);

        let mut break_points: Vec<BreakPoint> = config
            .breaks
            .iter()
            .map(|b| BreakPoint {
                note: b.note,
                frequency_ratio: b.frequency_ratio,
            })
            .collect();
        break_points.sort_by_key(|bp| bp.note);

        Self {
            waveform,
            amplitude_ratio: config.amplitude_ratio,
            chiff_intensity: config.chiff_intensity.unwrap_or(defaults.0),
            chiff_duration: config.chiff_duration.unwrap_or(defaults.1),
            attack_time: config.attack_time.unwrap_or(defaults.2),
            release_time: config.release_time.unwrap_or(defaults.3),
            break_points,
        }
    }

    pub fn resolve(&self, midi_note: u8) -> Stop {
        let frequency_ratio = self
            .break_points
            .iter()
            .rev()
            .find(|bp| midi_note >= bp.note)
            .map(|bp| bp.frequency_ratio)
            .unwrap_or(self.break_points[0].frequency_ratio);

        Stop {
            waveform: self.waveform,
            frequency_ratio,
            amplitude_ratio: self.amplitude_ratio,
            chiff_intensity: self.chiff_intensity,
            chiff_duration: self.chiff_duration,
            attack_time: self.attack_time,
            release_time: self.release_time,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopSpec {
    Fixed(Stop),
    Breaking(BreakingStop),
}

impl StopSpec {
    pub fn resolve(&self, midi_note: u8) -> Stop {
        match self {
            StopSpec::Fixed(s) => *s,
            StopSpec::Breaking(r) => r.resolve(midi_note),
        }
    }
}
