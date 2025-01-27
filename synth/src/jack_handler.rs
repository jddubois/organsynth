use crate::synth::Synth;
use jack::{AudioOut, Client, Port, ProcessHandler, ProcessScope};
use std::sync::{Arc, Mutex};

pub struct JackHandler {
    port: Port<AudioOut>,
    synth: Arc<Mutex<Synth>>,
}

impl JackHandler {
    pub fn new(port: Port<AudioOut>, synth: Arc<Mutex<Synth>>) -> Self {
        Self { port, synth }
    }
}

impl ProcessHandler for JackHandler {
    fn process(&mut self, _: &Client, ps: &ProcessScope) -> jack::Control {
        let mut synth = self.synth.lock().unwrap();
        let out: &mut [f32] = self.port.as_mut_slice(ps);
        out.iter_mut().for_each(|sample| {
            let s = synth.next_sample();
            if s > 1.0 {
                println!("Clipping: {}", s);
            }
            *sample = s.min(1.0);
        });
        jack::Control::Continue
    }
}
