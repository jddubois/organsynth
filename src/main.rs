mod jack_handler;
mod note;
mod oscillator;
mod reverb;
mod stop;
mod synth;
mod waveform;

use jack_handler::JackHandler;
use midir::{Ignore, MidiInput};
use std::sync::{Arc, Mutex};
use std::thread;
use synth::Synth;

fn midi_to_freq(midi_note: u8) -> f32 {
    440.0 * (2.0f32).powf((midi_note as f32 - 69.0) / 12.0)
}

/// Listen for MIDI input in a separate thread
fn start_midi_listener(
    synth: Arc<Mutex<Synth>>,
    port_num: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut midi_in = MidiInput::new("midir input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    if in_ports.is_empty() {
        eprintln!("No MIDI input ports available.");
        return Ok(());
    }

    let in_port = in_ports[port_num].clone();
    println!("Opening MIDI connection on port: {:?}", in_port.id());

    let synth_ref = Arc::clone(&synth);

    // Spawn the listener thread
    thread::spawn(move || {
        let _conn_in = midi_in
            .connect(
                &in_port,
                "midir-read-input",
                move |_timestamp, message, _| {
                    if message.len() == 3 {
                        let status = message[0] & 0xF0;
                        let data1 = message[1]; // MIDI note
                        let data2 = message[2]; // Velocity

                        match status {
                            0x90 => {
                                // Note On
                                if data2 > 0 {
                                    println!("Note On: {} {}", data1, data2);
                                    let freq = midi_to_freq(data1);
                                    let mut synth = synth_ref.lock().unwrap();
                                    synth.add_voice(freq);
                                } else {
                                    println!("Note Off: {} {}", data1, data2);
                                    // Treat Note On with velocity 0 as Note Off
                                    let freq = midi_to_freq(data1);
                                    let mut synth = synth_ref.lock().unwrap();
                                    synth.remove_voice(freq);
                                }
                            }
                            0x80 => {
                                // Note Off
                                let freq = midi_to_freq(data1);
                                let mut synth = synth_ref.lock().unwrap();
                                synth.remove_voice(freq);
                            }
                            _ => {}
                        }
                    }
                },
                (),
            )
            .expect("Failed to open MIDI connection");

        println!("MIDI thread running...");

        // Keep the MIDI connection open by blocking this thread
        loop {
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    Ok(())
}

// JACK handler for processing audio

// Main function to set up JACK client and start audio processing
fn main() {
    let (client, _status) =
        jack::Client::new("Synth", jack::ClientOptions::NO_START_SERVER).unwrap();

    let port = client
        .register_port("output", jack::AudioOut::default())
        .unwrap();

    let sample_rate = client.sample_rate() as f32;

    print!("Sample rate: {} Hz\n", sample_rate);

    let synth = Synth::new(sample_rate);

    // Create a MIDI input instance
    let mut midi_in = MidiInput::new("midir input").unwrap();
    // By default, ignore Active Sensing and Clock messages
    midi_in.ignore(Ignore::None);

    // List available input ports
    let in_ports = midi_in.ports();

    if in_ports.is_empty() {
        eprintln!("No MIDI input ports available.");
        return;
    }

    // Log all available input ports
    for (i, p) in in_ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p).unwrap());
    }

    let synth = Arc::new(Mutex::new(synth));
    let synth_clone = synth.clone();

    // start midi listener on every port, filtering by name

    for i in 0..in_ports.len() {
        let port_name = midi_in.port_name(&in_ports[i]).unwrap();
        if port_name.contains("Piano MIDI Device Digital Piano")
            || port_name.contains("pedalboard")
            || port_name.contains("teensy")
            || port_name.contains("IAC")
        {
            start_midi_listener(Arc::clone(&synth_clone), i).unwrap();
        }
    }

    let handler = JackHandler::new(port, synth);

    // Activate the JACK client
    let active_client = client.activate_async((), handler).unwrap();

    active_client
        .as_client()
        .connect_ports_by_name("Synth:output", "system:playback_1")
        .unwrap();
    active_client
        .as_client()
        .connect_ports_by_name("Synth:output", "system:playback_2")
        .unwrap();

    println!("Press ENTER to exit.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    active_client.deactivate().unwrap();
    println!("JACK client deactivated.");
}
