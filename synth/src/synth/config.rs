use super::stop::{BreakingStop, StopSpec};
use super::Stop;
use crate::config::SynthConfig;
use std::collections::HashMap;

pub fn get_stop(
    preset_stop_config: &crate::config::PresetStopConfig,
    config: &SynthConfig,
) -> StopSpec {
    match preset_stop_config {
        crate::config::PresetStopConfig::Named(name) => {
            resolve_stop_definition(&config.stops[name])
        }
        crate::config::PresetStopConfig::InlineBreaking(breaking) => {
            StopSpec::Breaking(BreakingStop::new(breaking))
        }
        crate::config::PresetStopConfig::Inline(stop) => StopSpec::Fixed(Stop::new(stop)),
    }
}

fn resolve_stop_definition(def: &crate::config::StopDefinition) -> StopSpec {
    match def {
        crate::config::StopDefinition::Simple(stop_config) => {
            StopSpec::Fixed(Stop::new(stop_config))
        }
        crate::config::StopDefinition::Breaking(breaking_config) => {
            StopSpec::Breaking(BreakingStop::new(breaking_config))
        }
    }
}

pub fn get_preset(
    preset_config: &crate::config::PresetConfig,
    config: &SynthConfig,
) -> Vec<StopSpec> {
    preset_config
        .stops
        .iter()
        .map(|stop| get_stop(stop, config))
        .collect()
}

pub fn get_preset_defaults(config: &SynthConfig) -> HashMap<u8, (u8, Vec<StopSpec>)> {
    config
        .preset_defaults
        .iter()
        .map(|preset_default| {
            (
                preset_default.midi_channel,
                (
                    config.presets[&preset_default.preset_name].midi_identifier,
                    get_preset(&config.presets[&preset_default.preset_name], config),
                ),
            )
        })
        .collect()
}

pub fn get_stops(config: &SynthConfig) -> HashMap<u8, StopSpec> {
    config
        .stops
        .values()
        .filter_map(|def| match def {
            crate::config::StopDefinition::Simple(stop_config) => {
                stop_config
                    .midi_identifier
                    .map(|id| (id, StopSpec::Fixed(Stop::new(stop_config))))
            }
            crate::config::StopDefinition::Breaking(breaking_config) => {
                breaking_config
                    .midi_identifier
                    .map(|id| (id, StopSpec::Breaking(BreakingStop::new(breaking_config))))
            }
        })
        .collect()
}

pub fn get_presets(config: &SynthConfig) -> HashMap<u8, Vec<StopSpec>> {
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
