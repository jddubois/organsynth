use crate::waveform::Waveform;  

#[derive(Copy, Clone)]
pub struct Stop {
    pub ratio: f32,
    pub amplitude: f32,
    pub waveform: Waveform,
}

impl Stop {
    pub fn new(ratio: f32, waveform: Waveform, amplitude: f32) -> Self {
        Self {
            ratio,
            amplitude,
            waveform,
        }
    }
}
