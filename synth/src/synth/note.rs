use super::oscillator::Oscillator;
use super::stop::Stop;

pub struct Note {
    sample_rate: f32,
    oscillators: Vec<Oscillator>,
    pub frequency: f32,
    pub is_released: bool,
}

impl Note {
    pub fn new(frequency: f32, sample_rate: f32, stops: &[Stop]) -> Self {
        let oscillators = stops
            .iter()
            .map(|stop| Oscillator::from_stop(stop, frequency, sample_rate))
            .collect();
        Self {
            sample_rate,
            oscillators,
            frequency,
            is_released: false,
        }
    }

    pub fn use_preset(&mut self, stops: &[Stop]) {
        self.oscillators = stops
            .iter()
            .map(|stop| Oscillator::from_stop(stop, self.frequency, self.sample_rate))
            .collect();
    }

    pub fn add_stop(&mut self, stop: &Stop) {
        self.oscillators.push(Oscillator::from_stop(
            stop,
            self.frequency,
            self.sample_rate,
        ));
    }

    pub fn remove_stop(&mut self, stop: &Stop) {
        for oscillator in &mut self.oscillators {
            if oscillator.matches_stop(stop, self.frequency) && !oscillator.is_released {
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
