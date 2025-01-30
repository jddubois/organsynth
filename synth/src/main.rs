mod config;
mod jack_handler;
mod midi;
mod synth;
use jack::MidiIn;
use jack_handler::JackHandler;
use std::sync::{Arc, Mutex};
use synth::Synth;

fn main() {
    let config: config::Config = config::load("../Config.toml").unwrap();
    let config::JackConfig {
        client_name,
        audio_out_port_name,
        midi_in_port_name,
        system_audio_l_port_name,
        system_audio_r_port_name,
    } = config.jack;
    let (client, _) =
        jack::Client::new(&client_name, jack::ClientOptions::NO_START_SERVER).unwrap();
    let sample_rate = client.sample_rate() as f32;
    let audio_out_port = client
        .register_port(&audio_out_port_name, jack::AudioOut::default())
        .unwrap();
    let midi_in_port = client
        .register_port(&midi_in_port_name, MidiIn::default())
        .unwrap();
    let synth = Arc::new(Mutex::new(Synth::new(sample_rate, config.synth)));
    let handler = JackHandler::new(synth.clone(), midi_in_port, audio_out_port);
    let active_client = client.activate_async((), handler).unwrap();
    let full_audio_out_port_name = format!("{}:{}", client_name, audio_out_port_name);
    let full_midi_in_port_name = format!("{}:{}", client_name, midi_in_port_name);
    active_client
        .as_client()
        .connect_ports_by_name(&full_audio_out_port_name, &system_audio_l_port_name)
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name(&full_audio_out_port_name, &system_audio_r_port_name)
        .unwrap();
    midi::MidiListener::new(active_client, full_midi_in_port_name)
        .start()
        .join()
        .unwrap();
}
