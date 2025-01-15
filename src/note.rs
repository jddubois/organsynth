use crate::oscillator::Oscillator;
use crate::stop::Stop;

pub struct Note {
    pub oscillators: Vec<Oscillator>,
    pub frequency: f32,
    pub is_released: bool,
}

impl Note {
    pub fn new(frequency: f32, sample_rate: f32, stops: &[Stop]) -> Self {
        let oscillators = stops
            .iter()
            .map(|stop| {
                Oscillator::new(
                    frequency * stop.ratio,
                    sample_rate,
                    stop.waveform,
                    stop.amplitude,
                )
            })
            .collect();
        Self {
            oscillators,
            frequency,
            is_released: false,
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
