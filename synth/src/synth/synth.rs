use super::waveform::Waveform;
use super::{config, Stop};
use crate::config::SynthConfig;
use crate::midi;
use crate::synth::thingy::InternalSynth;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

// TODO this file should still be cleaned up a bit
pub struct Synth {
    midi_tx: mpsc::Sender<[u8; 3]>,
    synths: Arc<Mutex<HashMap<u8, InternalSynth>>>,
}

impl Synth {
    pub fn new(sample_rate: f32, config: SynthConfig) -> Self {
        let (midi_tx, midi_rx) = mpsc::channel::<[u8; 3]>();
        let synths = Arc::new(Mutex::new(HashMap::new()));
        Self::spawn_midi_worker(
            synths.clone(),
            midi_rx,
            &config::get_stops(&config),
            &config::get_presets(&config),
            &config::get_preset_defaults(&config),
            sample_rate,
        );
        Self { midi_tx, synths }
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut synths = self.synths.lock().unwrap();
        synths
            .values_mut()
            .map(|synth| synth.next_sample())
            .sum::<f32>()
    }

    pub fn send_midi(&self, midi: [u8; 3]) {
        self.midi_tx.send(midi).unwrap();
    }

    fn spawn_midi_worker(
        synths: Arc<Mutex<HashMap<u8, InternalSynth>>>,
        midi_rx: mpsc::Receiver<[u8; 3]>,
        available_stops: &HashMap<u8, Stop>,
        available_presets: &HashMap<u8, Vec<Stop>>,
        preset_defaults: &HashMap<u8, Vec<Stop>>,
        sample_rate: f32,
    ) {
        let available_stops = available_stops.clone();
        let available_presets = available_presets.clone();
        let preset_defaults = preset_defaults.clone();
        std::thread::spawn(move || {
            for midi in midi_rx {
                match midi::try_parse(&midi) {
                    Ok(parsed) => {
                        let mut synths_guard = synths.lock().unwrap();
                        handle_midi_message(
                            &mut synths_guard,
                            &available_presets,
                            &available_stops,
                            &preset_defaults,
                            parsed,
                            sample_rate,
                        );
                    }
                    Err(e) => println!("Error parsing MIDI message: {:?}", e),
                }
            }
        });
    }
}

fn handle_midi_message(
    synths: &mut HashMap<u8, InternalSynth>,
    presets: &HashMap<u8, Vec<Stop>>,
    stops: &HashMap<u8, Stop>,
    preset_defaults: &HashMap<u8, Vec<Stop>>,
    message: midi::Message,
    sample_rate: f32,
) {
    let synth = get_or_create_synth(synths, message.channel, sample_rate, preset_defaults);
    match message.kind {
        midi::MessageKind::NoteOn => handle_note_on(synth, message),
        midi::MessageKind::NoteOff => handle_note_off(synth, message),
        midi::MessageKind::ControlChange => handle_control_change(synth, presets, stops, message),
        _ => {
            println!("Unhandled MIDI message: {:?}", message);
        }
    }
}

fn get_or_create_synth<'a>(
    synths: &'a mut HashMap<u8, InternalSynth>,
    channel: u8,
    sample_rate: f32,
    preset_defaults: &'a HashMap<u8, Vec<Stop>>,
) -> &'a mut InternalSynth {
    synths.entry(channel).or_insert_with(|| {
        let default_stops = vec![Stop {
            waveform: Waveform::Sine,
            frequency_ratio: 1.0,
            amplitude_ratio: 1.0,
        }];
        let stops = preset_defaults
            .get(&(channel + 1))
            .unwrap_or(&default_stops);
        InternalSynth::new(sample_rate, stops.clone())
    })
}

fn handle_note_on(synth: &mut InternalSynth, message: midi::Message) {
    let frequency = message.frequency().unwrap();
    if message.value == 0 {
        synth.remove_voice(frequency);
    } else {
        synth.add_voice(frequency);
    }
}

fn handle_note_off(synth: &mut InternalSynth, message: midi::Message) {
    let frequency = message.frequency().unwrap();
    synth.remove_voice(frequency);
}

fn handle_control_change(
    synth: &mut InternalSynth,
    presets: &HashMap<u8, Vec<Stop>>,
    stops: &HashMap<u8, Stop>,
    message: midi::Message,
) {
    if let Some(preset) = presets.get(&message.identifier).cloned() {
        println!("Using preset: {:?}", preset);
        synth.use_preset(preset.to_vec());
    } else if let Some(stop) = stops.get(&message.identifier) {
        if message.value == 0 {
            synth.remove_stop(*stop);
        } else {
            synth.add_stop(*stop);
        }
    }
    println!("Unhandled MIDI control change: {:?}", message);
}
