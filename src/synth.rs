use crate::note::Note;
use crate::reverb::Reverb;
use crate::stop::Stop;
use crate::waveform::Waveform;

struct LowPass {
    cutoff: f32,
    last_output: f32,
}

impl LowPass {
    pub fn new(cutoff: f32) -> Self {
        Self {
            cutoff,
            last_output: 0.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.last_output += self.cutoff * (input - self.last_output);
        self.last_output
    }
}

pub struct Synth {
    notes: Vec<Note>,
    sample_rate: f32,
    low_pass: LowPass,
    reverb: Reverb,
}

impl Synth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            notes: Vec::new(),
            sample_rate,
            low_pass: LowPass::new(1.0),
            reverb: Reverb::new(sample_rate),
        }
    }

    pub fn add_voice(&mut self, frequency: f32) {
        let note = Note::new(frequency, self.sample_rate, &default_stops());
        self.notes.push(note);
    }

    pub fn remove_voice(&mut self, frequency: f32) {
        for note in &mut self.notes {
            if note.frequency == frequency && !note.is_released {
                note.release();
                return;
            }
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.notes.retain_mut(|osc| !osc.is_finished());

        let mut sample = 0.0;
        for note in self.notes.iter_mut() {
            sample += note.next_sample();
        }

        let x = self.low_pass.process(sample) * 0.1;
        x
        // self.reverb.process_sample(x)
    }
}

fn default_stops() -> Vec<Stop> {
    vec![
        Stop::new(0.5, Waveform::Triangle, 0.6), // 16' Subbass
        Stop::new(1.0, Waveform::Triangle, 1.0), // 8' Principal
        Stop::new(2.0, Waveform::Triangle, 0.8), // 4' Octave
    ]
}
