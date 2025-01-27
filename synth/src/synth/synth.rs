use super::{config, Stop};
use crate::config::SynthConfig;
use crate::midi;
use crate::synth::thingy::InternalSynth;
use std::collections::HashMap;

pub struct Synth {
    sample_rate: f32,
    synths: HashMap<u8, InternalSynth>,
    available_stops: HashMap<u8, Stop>,
    available_presets: HashMap<u8, Vec<Stop>>,
    preset_defaults: HashMap<u8, Vec<Stop>>,
}

impl Synth {
    pub fn new(sample_rate: f32, config: SynthConfig) -> Self {
        Self {
            sample_rate,
            synths: HashMap::new(),
            preset_defaults: config::get_preset_defaults(&config),
            available_stops: config::get_stops(&config),
            available_presets: config::get_presets(&config),
        }
    }

    pub fn handle_midi_message(&mut self, message: midi::Message) {
        let midi::Message {
            kind,
            channel,
            identifier,
        } = message;

        let synth = self.synths.entry(channel).or_insert_with(|| {
            let stops = &self.preset_defaults[&(channel + 1)];
            InternalSynth::new(self.sample_rate, stops.clone())
        });
        match kind {
            midi::MessageKind::NoteOn => {
                let frequency = message.frequency().unwrap();
                synth.add_voice(frequency);
            }
            midi::MessageKind::NoteOff => {
                let frequency = message.frequency().unwrap();
                synth.remove_voice(frequency);
            }
            midi::MessageKind::ControlChange => {
                if let Some(preset) = self.available_presets.get(&identifier).cloned() {
                    println!("Using preset on channel {}", channel);
                    synth.use_preset(preset.to_vec());
                } else if let Some(stop) = self.available_stops.get(&identifier) {
                    println!("Using stop on channel {}", channel);
                    synth.add_stop(*stop);
                }
            }
            _ => {
                println!("Unhandled MIDI message: {:?}", message);
            }
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.synths
            .values_mut()
            .map(|synth| synth.next_sample())
            .sum::<f32>()
    }
}
