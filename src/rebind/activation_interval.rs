use egui::{Slider, Ui};
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ActivationIntervalParams {
    interval_start: f64,
    interval_end: f64,
    sustain: Option<f64>,

    #[serde(skip_serializing)]
    #[serde(default)]
    last_input: bool,

    #[serde(skip_serializing)]
    #[serde(default)]
    activation_start: f64,

    #[serde(skip_serializing)]
    #[serde(default)]
    activation_end: f64,
}

impl ActivationIntervalParams {
    pub fn new(activation_interval: Range<f64>, sustain: Option<f64>) -> Self {
        Self {
            interval_start: activation_interval.start,
            interval_end: activation_interval.end,
            sustain,
            ..Default::default()
        }
    }

    pub fn update(&mut self, state: bool, time: f64, use_sustain: bool) -> bool {
        let pressed_this_frame = state && !self.last_input;
        let released_this_frame = !state && self.last_input;
        self.last_input = state;

        if pressed_this_frame {
            self.activation_start = time;
        }
        if released_this_frame {
            self.activation_end = time;
        }

        let activation_length = time - self.activation_start;
        let activation_passed = time - self.activation_end;

        let activated_for_interval =
            activation_length >= self.interval_start && activation_length < self.interval_end;

        if state || !activated_for_interval {
            return false;
        }

        if use_sustain {
            if let Some(sustain) = self.sustain {
                activation_passed < sustain
            } else {
                released_this_frame
            }
        } else {
            released_this_frame
        }
    }

    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Interval start:");
                ui.add(Slider::new(&mut self.activation_start, 0.0..=10.0));
            });
            ui.horizontal(|ui| {
                ui.label("Interval end:");
                ui.add(Slider::new(&mut self.activation_end, 0.0..=10.0));
            });
            ui.horizontal(|ui| {
                ui.label("Sustain");
                match &mut self.sustain {
                    Some(val) => {
                        ui.add(Slider::new(val, 0.0..=1.0));
                    }
                    None => (),
                }
            });
        });
    }
}

impl Default for ActivationIntervalParams {
    fn default() -> Self {
        Self {
            activation_start: 0.0,
            activation_end: 0.0,
            last_input: false,
            interval_start: 0.5,
            interval_end: 1.0,
            sustain: Some(0.25),
        }
    }
}
