use crate::synth::Synth;
use jack::{AudioOut, Client, MidiIn, Port, ProcessHandler, ProcessScope};
use std::sync::{Arc, Mutex};

pub struct JackHandler {
    synth: Arc<Mutex<Synth>>,
    midi_in_port: Port<MidiIn>,
    audio_out_port: Port<AudioOut>,
}

impl JackHandler {
    pub fn new(
        synth: Arc<Mutex<Synth>>,
        midi_in_port: Port<MidiIn>,
        audio_out_port: Port<AudioOut>,
    ) -> Self {
        Self {
            synth,
            midi_in_port,
            audio_out_port,
        }
    }
}

impl ProcessHandler for JackHandler {
    fn process(&mut self, _: &Client, ps: &ProcessScope) -> jack::Control {
        let mut synth = self.synth.lock().unwrap();
        for event in self.midi_in_port.iter(ps) {
            if let Ok(midi) = <&[u8; 3]>::try_from(event.bytes) {
                synth.process_midi(*midi);
            }
        }
        let buffer = self.audio_out_port.as_mut_slice(ps);
        synth.fill_buffer(buffer);
        for sample in buffer.iter_mut() {
            *sample = sample.tanh();
        }
        jack::Control::Continue
    }
}
