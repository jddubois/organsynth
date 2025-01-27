use super::Stop;
use crate::config::SynthConfig;
use std::collections::HashMap;

pub fn get_stop(
    preset_stop_config: &crate::config::PresetStopConfig,
    config: &SynthConfig,
) -> Stop {
    match preset_stop_config {
        crate::config::PresetStopConfig::Named(name) => Stop::new(&config.stops[name]),
        crate::config::PresetStopConfig::Inline(stop) => Stop::new(stop),
    }
}

pub fn get_preset(preset_config: &crate::config::PresetConfig, config: &SynthConfig) -> Vec<Stop> {
    preset_config
        .stops
        .iter()
        .map(|stop| get_stop(stop, config))
        .collect()
}

pub fn get_preset_defaults(config: &SynthConfig) -> HashMap<u8, Vec<Stop>> {
    config
        .preset_defaults
        .iter()
        .map(|preset_default| {
            (
                preset_default.midi_channel,
                get_preset(&config.presets[&preset_default.preset_name], config),
            )
        })
        .collect()
}

pub fn get_stops(config: &SynthConfig) -> HashMap<u8, Stop> {
    config
        .stops
        .values()
        .filter(|stop_config| stop_config.midi_identifier.is_some())
        .map(|stop_config| (stop_config.midi_identifier.unwrap(), Stop::new(stop_config)))
        .collect()
}

pub fn get_presets(config: &SynthConfig) -> HashMap<u8, Vec<Stop>> {
    config
        .presets
        .values()
        .map(|preset_config| {
            (
                preset_config.midi_identifier,
                get_preset(preset_config, config),
            )
        })
        .collect()
}
