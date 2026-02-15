use super::stop::StopSpec;
use super::waveform::Waveform;
use super::{config, Stop};
use crate::config::{ReverbConfig, SynthConfig};
use crate::midi;
use crate::synth::thingy::InternalSynth;
use std::collections::HashMap;

pub struct Synth {
    synths: HashMap<u8, InternalSynth>,
    available_stops: HashMap<u8, StopSpec>,
    available_presets: HashMap<u8, Vec<StopSpec>>,
    preset_defaults: HashMap<u8, (u8, Vec<StopSpec>)>,
    sample_rate: f32,
    reverb_config: ReverbConfig,
}

impl Synth {
    pub fn new(sample_rate: f32, config: SynthConfig) -> Self {
        let reverb_config = config.reverb.clone().unwrap_or_default();
        Self {
            synths: HashMap::new(),
            available_stops: config::get_stops(&config),
            available_presets: config::get_presets(&config),
            preset_defaults: config::get_preset_defaults(&config),
            sample_rate,
            reverb_config,
        }
    }

    pub fn process_midi(&mut self, midi: [u8; 3]) {
        let message = match midi::try_parse(&midi) {
            Ok(m) => m,
            Err(_) => return,
        };

        self.ensure_synth(message.channel);

        match message.kind {
            midi::MessageKind::NoteOn => {
                let synth = self.synths.get_mut(&message.channel).unwrap();
                let frequency = message.frequency().unwrap();
                if message.value == 0 {
                    synth.remove_voice(frequency);
                } else {
                    synth.add_voice(frequency, message.identifier);
                }
            }
            midi::MessageKind::NoteOff => {
                let synth = self.synths.get_mut(&message.channel).unwrap();
                let frequency = message.frequency().unwrap();
                synth.remove_voice(frequency);
            }
            midi::MessageKind::ControlChange => {
                eprintln!(
                    "[CC] channel={} cc={} value={}",
                    message.channel, message.identifier, message.value
                );
                if let Some(preset_stops) =
                    self.available_presets.get(&message.identifier).cloned()
                {
                    let synth = self.synths.get_mut(&message.channel).unwrap();
                    if message.value == 0 {
                        eprintln!("[CC] deactivating preset cc={}", message.identifier);
                        synth.deactivate_preset(message.identifier);
                    } else {
                        eprintln!(
                            "[CC] activating preset cc={} ({} stops)",
                            message.identifier,
                            preset_stops.len()
                        );
                        synth.activate_preset(message.identifier, preset_stops);
                    }
                } else if let Some(stop) = self.available_stops.get(&message.identifier).cloned() {
                    let synth = self.synths.get_mut(&message.channel).unwrap();
                    if message.value == 0 {
                        synth.remove_stop(&stop);
                    } else {
                        synth.add_stop(stop);
                    }
                } else {
                    eprintln!("[CC] unrecognized cc={}, ignoring", message.identifier);
                }
            }
            _ => {}
        }
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self
                .synths
                .values_mut()
                .map(|synth| synth.next_sample())
                .sum();
        }
    }

    fn ensure_synth(&mut self, channel: u8) {
        if self.synths.contains_key(&channel) {
            return;
        }
        let (default_cc_id, stops) = self
            .preset_defaults
            .get(&(channel + 1))
            .cloned()
            .unwrap_or_else(|| {
                (
                    0,
                    vec![StopSpec::Fixed(Stop {
                        waveform: Waveform::Sine,
                        frequency_ratio: 1.0,
                        amplitude_ratio: 1.0,
                        chiff_intensity: 0.1,
                        chiff_duration: 0.05,
                        attack_time: 0.05,
                        release_time: 0.1,
                    })],
                )
            });
        eprintln!(
            "[INIT] channel={} default_cc_id={} stops={}",
            channel,
            default_cc_id,
            stops.len()
        );
        let synth = InternalSynth::new(self.sample_rate, default_cc_id, stops, &self.reverb_config);
        self.synths.insert(channel, synth);
    }
}
