mod config;
mod jack_handler;
mod midi;
mod synth;
use jack_handler::JackHandler;
use std::sync::{Arc, Mutex};
use synth::Synth;

fn init_jack_synth(
    config: config::Config,
) -> Result<(Arc<Mutex<Synth>>, jack::AsyncClient<(), JackHandler>), jack::Error> {
    let config::JackConfig {
        client_name,
        port_name,
        destination_l_port_name,
        destination_r_port_name,
    } = config.jack;
    let (client, _) = jack::Client::new(&client_name, jack::ClientOptions::NO_START_SERVER)?;
    let port = client.register_port(&port_name, jack::AudioOut::default())?;
    let sample_rate = client.sample_rate() as f32;
    let synth = Arc::new(Mutex::new(Synth::new(sample_rate, config.synth)));
    let handler = JackHandler::new(port, synth.clone());
    let active_client = client.activate_async((), handler)?;
    let output_port_name = format!("{}:{}", client_name, port_name);
    println!("Connecting to {}", output_port_name);
    active_client
        .as_client()
        .connect_ports_by_name(&output_port_name, &destination_l_port_name)?;
    active_client
        .as_client()
        .connect_ports_by_name(&output_port_name, &destination_r_port_name)?;
    Ok((synth, active_client))
}

fn main() {
    let config: config::Config = config::load("Config.toml").unwrap();
    println!("Config: {:?}", config);
    let (synth, _active_client) = init_jack_synth(config).unwrap();
    let midi_listener = midi::MidiListener::new(synth);
    midi_listener.start().join().unwrap();
}
