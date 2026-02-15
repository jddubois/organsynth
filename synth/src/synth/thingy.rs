use super::{filters::Filter, note::Note, stop::StopSpec};
use crate::config::ReverbConfig;
use std::collections::HashMap;

pub struct InternalSynth {
    sample_rate: f32,
    filters: Vec<Box<dyn Filter>>,
    active_presets: HashMap<u8, Vec<StopSpec>>,
    notes: Vec<Note>,
    // Diagnostics
    diag_counter: u32,
    diag_interval: u32,
    diag_pre_peak: f32,
    diag_post_peak: f32,
    diag_nan_count: u32,
}

impl InternalSynth {
    pub fn new(
        sample_rate: f32,
        default_cc_id: u8,
        stops: Vec<StopSpec>,
        reverb_config: &ReverbConfig,
    ) -> Self {
        let mut active_presets = HashMap::new();
        active_presets.insert(default_cc_id, stops);
        Self {
            notes: Vec::new(),
            sample_rate,
            filters: vec![
                Box::new(super::filters::LowPass::new(0.7)),
                Box::new(super::filters::LowPass::new(0.7)),
                Box::new(super::filters::Freeverb::new(sample_rate, reverb_config)),
            ],
            active_presets,
            diag_counter: 0,
            diag_interval: sample_rate as u32,
            diag_pre_peak: 0.0,
            diag_post_peak: 0.0,
            diag_nan_count: 0,
        }
    }

    fn current_stops(&self) -> Vec<StopSpec> {
        let mut stops = Vec::new();
        for preset_stops in self.active_presets.values() {
            for stop in preset_stops {
                if !stops.contains(stop) {
                    stops.push(stop.clone());
                }
            }
        }
        stops
    }

    pub fn add_voice(&mut self, frequency: f32, midi_note: u8) {
        let stops = self.current_stops();
        let note = Note::new(frequency, midi_note, self.sample_rate, &stops);
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

    pub fn activate_preset(&mut self, cc_id: u8, stops: Vec<StopSpec>) {
        let existing_stops = self.current_stops();
        self.active_presets.insert(cc_id, stops.clone());

        let mut added = 0;
        for stop in &stops {
            if !existing_stops.contains(stop) {
                added += 1;
                for note in &mut self.notes {
                    note.add_stop(stop);
                }
            }
        }
        eprintln!(
            "[PRESET] activated cc={}, {} total stops, {} newly added, active_presets: {:?}",
            cc_id,
            stops.len(),
            added,
            self.active_presets.keys().collect::<Vec<_>>()
        );
    }

    pub fn deactivate_preset(&mut self, cc_id: u8) {
        let removed_stops = match self.active_presets.remove(&cc_id) {
            Some(stops) => stops,
            None => {
                eprintln!("[PRESET] deactivate cc={}: not found in active_presets", cc_id);
                return;
            }
        };

        let removed_count = removed_stops.len();
        let mut to_remove = removed_stops;
        for remaining in self.active_presets.values() {
            for stop in remaining {
                if let Some(pos) = to_remove.iter().position(|s| s == stop) {
                    to_remove.remove(pos);
                }
            }
        }

        eprintln!(
            "[PRESET] deactivated cc={}, removed {} stops ({} were shared), active_presets: {:?}",
            cc_id,
            to_remove.len(),
            removed_count - to_remove.len(),
            self.active_presets.keys().collect::<Vec<_>>()
        );
        for stop in &to_remove {
            for note in &mut self.notes {
                note.remove_stop(stop);
            }
        }
    }

    pub fn add_stop(&mut self, stop: StopSpec) {
        for note in &mut self.notes {
            note.add_stop(&stop);
        }
    }

    pub fn remove_stop(&mut self, stop: &StopSpec) {
        for note in &mut self.notes {
            note.remove_stop(stop);
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.notes.retain_mut(|note| !note.is_finished());
        let mut sample = 0.0;
        for note in self.notes.iter_mut() {
            sample += note.next_sample();
        }
        sample *= 0.05;

        // Track pre-filter peak
        let pre_abs = sample.abs();
        if pre_abs > self.diag_pre_peak {
            self.diag_pre_peak = pre_abs;
        }
        if !sample.is_finite() {
            self.diag_nan_count += 1;
            sample = 0.0;
        }

        for filter in self.filters.iter_mut() {
            sample = filter.process(sample);
        }

        // Track post-filter peak
        let post_abs = sample.abs();
        if post_abs > self.diag_post_peak {
            self.diag_post_peak = post_abs;
        }

        // Periodic diagnostic logging (~once per second)
        self.diag_counter += 1;
        if self.diag_counter >= self.diag_interval {
            let note_count = self.notes.len();
            let osc_count: usize = self.notes.iter().map(|n| n.oscillator_count()).sum();
            if note_count > 0 || self.diag_pre_peak > 0.01 {
                eprintln!(
                    "[DIAG] notes={} oscs={} pre_peak={:.4} post_peak={:.4} nan_count={}",
                    note_count, osc_count, self.diag_pre_peak, self.diag_post_peak, self.diag_nan_count,
                );
            }
            self.diag_counter = 0;
            self.diag_pre_peak = 0.0;
            self.diag_post_peak = 0.0;
            self.diag_nan_count = 0;
        }

        sample
    }
}
