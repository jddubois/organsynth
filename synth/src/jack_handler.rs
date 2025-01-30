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
        self.midi_in_port
            .iter(ps)
            .for_each(|event: jack::RawMidi<'_>| {
                if let Ok(midi) = <&[u8; 3]>::try_from(event.bytes) {
                    synth.send_midi(*midi);
                }
            });
        self.audio_out_port
            .as_mut_slice(ps)
            .iter_mut()
            .for_each(|sample| {
                *sample = synth.next_sample();
            });
        jack::Control::Continue
    }
}
