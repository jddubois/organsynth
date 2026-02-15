use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub jack: JackConfig,
    pub synth: SynthConfig,
}

#[derive(Debug, Deserialize)]
pub struct SynthConfig {
    pub stops: HashMap<String, StopConfig>,
    pub presets: HashMap<String, PresetConfig>,
    pub preset_defaults: Vec<PresetDefaultConfig>,
    pub reverb: Option<ReverbConfig>,
}

#[derive(Debug, Deserialize)]
pub struct JackConfig {
    pub client_name: String,
    pub audio_out_port_name: String,
    pub midi_in_port_name: String,
    pub system_audio_l_port_name: String,
    pub system_audio_r_port_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PresetConfig {
    pub midi_identifier: u8,
    pub stops: Vec<PresetStopConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PresetStopConfig {
    Named(String),
    Inline(StopConfig),
}

#[derive(Debug, Deserialize)]
pub struct StopConfig {
    pub midi_identifier: Option<u8>,
    pub waveform: String,
    pub frequency_ratio: f32,
    pub amplitude_ratio: f32,
    pub chiff_intensity: Option<f32>,
    pub chiff_duration: Option<f32>,
    pub attack_time: Option<f32>,
    pub release_time: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct PresetDefaultConfig {
    pub midi_channel: u8,
    pub preset_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReverbConfig {
    pub room_size: f32,
    pub damping: f32,
    pub wet: f32,
    pub dry: f32,
    pub pre_delay_ms: Option<f32>,
}

impl Default for ReverbConfig {
    fn default() -> Self {
        Self {
            room_size: 0.85,
            damping: 0.5,
            wet: 0.35,
            dry: 0.65,
            pre_delay_ms: Some(20.0),
        }
    }
}
