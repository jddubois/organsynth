use crate::jack_handler::JackHandler;
use jack::{Port, PortFlags, Unowned};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const MIDI_PORT_TYPE: &str = "8 bit raw midi";
const SYSTEM_PORT_PREFIX: &str = "system:";

pub struct MidiListener {
    jack_client: jack::AsyncClient<(), JackHandler>,
    jack_midi_in_port_name: String,
    active_port_names: Arc<Mutex<HashSet<String>>>,
}

impl MidiListener {
    pub fn new(
        jack_client: jack::AsyncClient<(), JackHandler>,
        jack_midi_in_port_name: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            jack_client,
            jack_midi_in_port_name,
            active_port_names: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub fn start(self: Arc<Self>) -> thread::JoinHandle<()> {
        let clone = self.clone();
        thread::spawn(move || loop {
            if let Err(e) = clone.poll() {
                println!("Error polling MIDI ports: {:?}", e);
            }
            thread::sleep(Duration::from_secs(1));
        })
    }

    fn poll(&self) -> Result<(), jack::Error> {
        let ports: Vec<Port<Unowned>> = self
            .jack_client
            .as_client()
            .ports(None, Some(MIDI_PORT_TYPE), PortFlags::IS_OUTPUT)
            .iter()
            .filter(|name| !name.starts_with(SYSTEM_PORT_PREFIX))
            .filter_map(|name| self.jack_client.as_client().port_by_name(name))
            .collect();
        let mut active_port_names = self.active_port_names.lock().unwrap();
        for port in ports.iter() {
            let port_name = port.name().unwrap();
            if active_port_names.contains(&port_name) {
                continue;
            }
            println!("Connecting to MIDI port: {}", port_name);
            self.jack_client
                .as_client()
                .connect_ports_by_name(&port.name()?, &self.jack_midi_in_port_name)?;
        }
        *active_port_names = ports.iter().map(|port| port.name().unwrap()).collect();
        Ok(())
    }
}
