use std::time::Instant;

#[derive(Debug, PartialEq, Clone)]
pub struct ActivationIntervalParams {
    pub activation_start: Instant,
    pub activation_end: Instant,
    pub last_input: bool,
    pub interval_start: f32,
    pub interval_end: f32,
    pub sustain: Option<f32>,
}

impl ActivationIntervalParams {
    pub fn update(&mut self, state: bool) -> bool {
        let pressed_this_frame = state && !self.last_input;
        let released_this_frame = !state && self.last_input;
        self.last_input = state;

        if pressed_this_frame {
            self.activation_start = Instant::now();
        }
        if released_this_frame {
            self.activation_end = Instant::now();
        }

        let activation_length = self.activation_start.elapsed().as_secs_f32();
        let activation_passed = self.activation_end.elapsed().as_secs_f32();

        let activated_for_interval =
            activation_length >= self.interval_start && activation_length < self.interval_end;

        if state || !activated_for_interval {
            return false;
        }

        if let Some(sustain) = self.sustain {
            activation_passed < sustain // output true while < sustain
        } else {
            released_this_frame // output true for one frame
        }
    }
}

impl Default for ActivationIntervalParams {
    fn default() -> Self {
        Self {
            activation_start: Instant::now(),
            activation_end: Instant::now(),
            last_input: false,
            interval_start: 0.5,
            interval_end: 1.0,
            sustain: Some(0.25),
        }
    }
}
