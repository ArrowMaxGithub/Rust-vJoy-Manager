use egui::{Checkbox, Slider, Ui};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};
use vjoy::Axis;

use super::{TABLE_COLUMN_LEFT_WIDTH, TABLE_ROW_HEIGHT};

/// Activation type and conditions for two input button to single output axis rebinds.
///
/// ## Examples usages
/// - Rebind '+' and '-' to 'throttle axis' with absolute reponse --> 100% when '+' is pressed, -100% when '-' is pressed, 0% if neither is pressed.
/// - Rebind 'W' and 'S' to 'pitch axis' with a linear response and return to zero --> Increase when 'W' is pressed, decrease when 'S' is pressed, return to zero linearly if neither is pressed.
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
pub enum TwoButtonsToAxisModifier {
    /// Buttons map to absoulte min/max values, neutral when neither is pressed.
    Absolute,
    /// Linear coefficient, keep value or return to zero when no button is pressed.
    Linear { coefficient: f64, keep_value: bool },
    Click {
        coefficient: f64,

        #[serde(skip_serializing)]
        #[serde(default)]
        last_input_neg: bool,

        #[serde(skip_serializing)]
        #[serde(default)]
        last_input_pos: bool,
    },
}

impl Default for TwoButtonsToAxisModifier {
    fn default() -> Self {
        Self::Absolute
    }
}

impl TwoButtonsToAxisModifier {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| match self {
            TwoButtonsToAxisModifier::Absolute => {}
            TwoButtonsToAxisModifier::Linear {
                coefficient,
                keep_value,
            } => {
                TableBuilder::new(ui)
                    .column(Column::exact(TABLE_COLUMN_LEFT_WIDTH))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Coefficient:");
                            });
                            row.col(|ui| {
                                ui.push_id("Coefficient", |ui| {
                                    ui.add(Slider::new(coefficient, 0.0..=10.0));
                                });
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Keep value:");
                            });
                            row.col(|ui| {
                                ui.push_id("KeepValue", |ui| {
                                    ui.add(Checkbox::new(keep_value, ""));
                                });
                            });
                        });
                    });
            }
            TwoButtonsToAxisModifier::Click {
                coefficient,
                last_input_neg: _,
                last_input_pos: _,
            } => {
                TableBuilder::new(ui)
                    .column(Column::exact(TABLE_COLUMN_LEFT_WIDTH))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Coefficient:");
                            });
                            row.col(|ui| {
                                ui.add(Slider::new(coefficient, 0.0..=1.0));
                            });
                        });
                    });
            }
        });
    }
}

// output range 0..=32767
pub fn apply_two_buttons_to_axis_modifier(
    input_neg: bool,
    input_pos: bool,
    output: &Axis,
    modifier: &mut TwoButtonsToAxisModifier,
    delta_t: f64,
) -> i32 {
    let value = match modifier {
        TwoButtonsToAxisModifier::Absolute => match (input_neg, input_pos) {
            (true, false) => 0,
            (true, true) => 16384,
            (false, false) => 16384,
            (false, true) => 32767,
        },
        TwoButtonsToAxisModifier::Linear {
            coefficient,
            keep_value,
        } => {
            let mut current_output_value = output.get();
            let min_change = 1;
            let delta = (*coefficient * 32767.0 * delta_t) as i32;
            match (input_neg, input_pos) {
                (true, false) => {
                    current_output_value -= delta.max(min_change);
                }
                (false, true) => {
                    current_output_value += delta.max(min_change);
                }
                (false, false) => {
                    if *keep_value {
                        return current_output_value;
                    }

                    let center_distance = 16384_i32.abs_diff(current_output_value);
                    if center_distance <= delta as u32 {
                        current_output_value = 16384_i32;
                    } else {
                        current_output_value -=
                            delta.max(min_change) * (current_output_value - 16384).signum();
                    }
                }
                (true, true) => (),
            }

            current_output_value
        }

        TwoButtonsToAxisModifier::Click {
            coefficient,
            last_input_neg,
            last_input_pos,
        } => {
            let mut current_output_value = output.get();
            let min_change = 1;
            let delta = (*coefficient * 32767.0) as i32;

            let should_click_neg = input_neg && input_neg != *last_input_neg;
            if should_click_neg {
                current_output_value -= delta.max(min_change);
            }

            let should_click_pos = input_pos && input_pos != *last_input_pos;
            if should_click_pos {
                current_output_value += delta.max(min_change);
            }

            *last_input_neg = input_neg;
            *last_input_pos = input_pos;

            current_output_value
        }
    };

    value.clamp(0, 32767)
}
