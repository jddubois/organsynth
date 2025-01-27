use super::try_parse;
use crate::synth::Synth;
use midir::{MidiInput, MidiInputConnection, MidiInputPort};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

// TODO maybe move to config
const MIDI_INPUT_NAME: &str = "midir input";

pub struct MidiPortListener {
    synth: Arc<Mutex<Synth>>,
    port: MidiInputPort,
    stop: Arc<AtomicBool>,
}

impl MidiPortListener {
    pub fn new(synth: Arc<Mutex<Synth>>, port: MidiInputPort) -> Arc<Self> {
        Arc::new(Self {
            synth,
            port,
            stop: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(self: Arc<Self>) {
        thread::spawn(move || {
            let _connection = self.connect();
            let port_name = self.port_name();
            println!("Starting MIDI thread for {:?}", port_name);
            loop {
                if self.stop.load(Ordering::SeqCst) {
                    println!("Stopping MIDI thread for {:?}", port_name);
                    break;
                }
                thread::sleep(Duration::from_millis(1000));
            }
        });
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    fn connect(&self) -> MidiInputConnection<()> {
        let midi_in: MidiInput = MidiInput::new(MIDI_INPUT_NAME).unwrap();
        let synth = self.synth.clone();
        let port_name = self.port_name();
        midi_in
            .connect(
                &self.port,
                &self.port_name(),
                move |_x, raw_message, _| {
                    handle_raw_message(&port_name, synth.clone(), raw_message);
                },
                (),
            )
            .unwrap()
    }

    fn port_name(&self) -> String {
        MidiInput::new(MIDI_INPUT_NAME)
            .unwrap()
            .port_name(&self.port)
            .unwrap()
    }
}

fn handle_raw_message(port_name: &str, synth: Arc<Mutex<Synth>>, raw_message: &[u8]) {
    let message_result = try_parse(raw_message);
    if message_result.is_err() {
        println!(
            "Error parsing MIDI message: {:?}",
            message_result.err().unwrap()
        );
        return;
    }
    let message = message_result.unwrap();
    println!("MIDI message: {:?} on port {}", message, port_name);
    let mut synth = synth.lock().unwrap();
    synth.handle_midi_message(message);
}
