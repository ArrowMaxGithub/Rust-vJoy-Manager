use egui::{Slider, Ui};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    AsRefStr,
    EnumIter,
    EnumString,
    EnumVariantNames,
)]
#[serde(tag = "modifier")]
pub enum VirtualAxisTrimModifier {
    Click {
        #[serde(flatten)]
        params: VirtualAxisTrimParams,
    },
    Linear {
        #[serde(flatten)]
        params: VirtualAxisTrimParams,
    },
}

impl Default for VirtualAxisTrimModifier {
    fn default() -> Self {
        Self::Click {
            params: Default::default(),
        }
    }
}

impl VirtualAxisTrimModifier {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| match self {
            VirtualAxisTrimModifier::Click { params } => {
                params.widget(ui);
            }

            VirtualAxisTrimModifier::Linear { params } => {
                params.widget(ui);
            }
        });
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct VirtualAxisTrimParams {
    value_normalized: f64,

    #[serde(skip_serializing)]
    #[serde(default)]
    accumulated: f64,

    #[serde(skip_serializing)]
    #[serde(default)]
    last_input_neg: bool,

    #[serde(skip_serializing)]
    #[serde(default)]
    last_input_pos: bool,
}

impl VirtualAxisTrimParams {
    pub fn new(value_normalized: f64) -> Self {
        Self {
            value_normalized,
            ..Default::default()
        }
    }

    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Value:");
                ui.add(Slider::new(&mut self.value_normalized, 0.0..=1.0));
            });
        });
    }
}

pub fn apply_virtual_axis_trim_modifier(
    input: i32,
    trim_neg: bool,
    trim_pos: bool,
    trim_reset: bool,
    delta_t: f64,
    modifier: &mut VirtualAxisTrimModifier,
) -> i32 {
    let mut axis_normalized_value = input as f64 / 32767.0;
    match modifier {
        VirtualAxisTrimModifier::Click { params } => {
            let should_trim_neg = trim_neg && trim_neg != params.last_input_neg;
            if should_trim_neg {
                params.accumulated -= params.value_normalized;
            }

            let should_trim_pos = trim_pos && trim_pos != params.last_input_pos;
            if should_trim_pos {
                params.accumulated += params.value_normalized;
            }

            params.accumulated = params.accumulated.clamp(-1.0, 1.0);
            if trim_reset {
                params.accumulated = 0.0;
            }
            params.last_input_neg = trim_neg;
            params.last_input_pos = trim_pos;
            axis_normalized_value += params.accumulated;
        }

        VirtualAxisTrimModifier::Linear { params } => {
            if trim_neg {
                params.accumulated -= params.value_normalized * delta_t;
            }

            if trim_pos {
                params.accumulated += params.value_normalized * delta_t;
            }

            params.accumulated = params.accumulated.clamp(-1.0, 1.0);
            if trim_reset {
                params.accumulated = 0.0;
            }
            axis_normalized_value += params.accumulated;
        }
    }

    (axis_normalized_value * 32767.0)
        .abs()
        .clamp(0.0, 32767.0)
        .floor() as i32
}
