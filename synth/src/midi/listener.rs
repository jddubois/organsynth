use super::port_listener::MidiPortListener;
use crate::synth;
use midir::{MidiInput, MidiInputPort};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// TODO move to common
const MIDI_INPUT_NAME: &str = "midir input";

pub struct MidiListener {
    synth: Arc<Mutex<synth::Synth>>,
    listeners: Arc<Mutex<HashMap<String, Arc<MidiPortListener>>>>,
}

impl MidiListener {
    pub fn new(synth: Arc<Mutex<synth::Synth>>) -> Arc<Self> {
        Arc::new(Self {
            synth,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn start(self: Arc<Self>) -> thread::JoinHandle<()> {
        let clone = self.clone();
        thread::spawn(move || loop {
            clone.poll();
            thread::sleep(Duration::from_secs(1));
        })
    }

    fn poll(&self) {
        let midi_in = MidiInput::new(MIDI_INPUT_NAME).unwrap();
        let in_ports = midi_in.ports();
        self.add_new_ports(in_ports.clone());
        self.remove_old_ports(in_ports.clone());
    }

    fn add_new_ports(&self, in_ports: Vec<MidiInputPort>) {
        let mut listeners = self.listeners.lock().unwrap();
        for port in in_ports.iter() {
            listeners.entry(port.id()).or_insert_with(|| {
                let listener = MidiPortListener::new(self.synth.clone(), port.clone());
                listener.clone().start();
                listener
            });
        }
    }

    fn remove_old_ports(&self, in_ports: Vec<MidiInputPort>) {
        let mut listeners = self.listeners.lock().unwrap();
        let in_port_ids: HashSet<String> = in_ports.iter().map(|port| port.id()).collect();
        listeners.retain(|id, listener| {
            if !in_port_ids.contains(id) {
                listener.stop();
                false
            } else {
                true
            }
        });
    }
}
