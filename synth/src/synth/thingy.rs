use super::{filters::Filter, note::Note, stop::Stop, waveform::Waveform};

pub struct InternalSynth {
    sample_rate: f32,
    filters: Vec<Box<dyn Filter>>,
    stops: Vec<Stop>,
    notes: Vec<Note>,
}

impl InternalSynth {
    pub fn new(sample_rate: f32, stops: Vec<Stop>) -> Self {
        Self {
            notes: Vec::new(),
            sample_rate,
            filters: vec![
                Box::new(super::filters::LowPass::new(0.1)),
                Box::new(super::filters::SimpleReverb::new(
                    sample_rate,
                    100.0,
                    0.5,
                    0.5,
                )),
            ],
            stops,
        }
    }

    pub fn add_voice(&mut self, frequency: f32) {
        let note = Note::new(frequency, self.sample_rate, &self.stops);
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

    pub fn use_preset(&mut self, stops: Vec<Stop>) {
        self.stops = stops;
        for note in &mut self.notes {
            note.use_preset(&self.stops);
        }
    }

    pub fn add_stop(&mut self, stop: Stop) {
        self.stops.push(stop);
        for note in &mut self.notes {
            note.add_stop(&stop);
        }
    }

    pub fn remove_stop(&mut self, stop: Stop) {
        let position = self.stops.iter().position(|s| s == &stop);
        if let Some(position) = position {
            self.stops.remove(position);
        }
        for note in &mut self.notes {
            note.remove_stop(&stop);
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.notes.retain_mut(|note| !note.is_finished());
        let mut sample = 0.0;
        for note in self.notes.iter_mut() {
            sample += note.next_sample();
        }
        for filter in self.filters.iter_mut() {
            sample = filter.process(sample);
        }
        sample * 0.1
    }
}
