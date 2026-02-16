use super::oscillator::Oscillator;
use super::stop::StopSpec;

pub struct Note {
    sample_rate: f32,
    oscillators: Vec<Oscillator>,
    pub frequency: f32,
    pub midi_note: u8,
    pub is_released: bool,
}

impl Note {
    pub fn new(frequency: f32, midi_note: u8, sample_rate: f32, stops: &[StopSpec]) -> Self {
        let oscillators = stops
            .iter()
            .map(|spec| {
                let stop = spec.resolve(midi_note);
                Oscillator::from_stop(&stop, frequency, sample_rate)
            })
            .collect();
        Self {
            sample_rate,
            oscillators,
            frequency,
            midi_note,
            is_released: false,
        }
    }

    pub fn add_stop(&mut self, spec: &StopSpec) {
        let stop = spec.resolve(self.midi_note);
        let mut osc = Oscillator::from_stop(&stop, self.frequency, self.sample_rate);
        if self.is_released {
            osc.release();
        }
        self.oscillators.push(osc);
    }

    pub fn remove_stop(&mut self, spec: &StopSpec) {
        let stop = spec.resolve(self.midi_note);
        for oscillator in &mut self.oscillators {
            if oscillator.matches_stop(&stop, self.frequency) && !oscillator.is_released {
                oscillator.release();
                return;
            }
        }
    }

    pub fn release(&mut self) {
        self.is_released = true;
        self.oscillators
            .iter_mut()
            .for_each(|oscillator| oscillator.release());
    }

    pub fn oscillator_count(&self) -> usize {
        self.oscillators.len()
    }

    pub fn is_finished(&self) -> bool {
        self.oscillators.iter().all(|osc| osc.is_finished())
    }

    pub fn next_sample(&mut self) -> f32 {
        self.oscillators
            .iter_mut()
            .map(|osc| osc.next_sample())
            .sum()
    }
}
