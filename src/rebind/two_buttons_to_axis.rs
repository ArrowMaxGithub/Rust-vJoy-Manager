use serde::{Deserialize, Serialize};
use vjoy::Axis;

/// Activation type and conditions for two input button to single output axis rebinds
///
/// ## Examples usages
/// - Rebind '+' and '-' to 'throttle axis' with absolute reponse --> 100% when '+' is pressed, -100% when '-' is pressed, 0% if neither is pressed
/// - Rebind 'W' and 'S' to 'pitch axis' with a linear response and return to zero --> Increase when 'W' is pressed, decrease when 'S' is pressed, return to zero linearly if neither is pressed
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "modifier")]
pub enum TwoButtonsToAxisModifier {
    /// Buttons map to absoulte min/max values, neutral when neither is pressed
    Absolute,
    /// Linear coefficient, keep value or return to zero when no button is pressed
    Linear { coefficient: f64, keep_value: bool },
    /// Exponential coefficient, keep value or return to zero when no button is pressed
    Exponential { coefficient: f64, keep_value: bool },
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
        TwoButtonsToAxisModifier::Exponential {
            coefficient,
            keep_value,
        } => {
            let mut current_output_value = output.get();
            let min_change = 1;
            let delta_min =
                (*coefficient * (0_i32.abs_diff(current_output_value)) as f64 * delta_t) as i32;
            let delta_max =
                (*coefficient * (32767_i32.abs_diff(current_output_value)) as f64 * delta_t) as i32;

            match (input_neg, input_pos) {
                (true, false) => {
                    current_output_value -= delta_min.max(min_change);
                }
                (false, true) => {
                    current_output_value += delta_max.max(min_change);
                }
                (false, false) => {
                    if *keep_value {
                        return current_output_value;
                    }

                    let center_distance = 16384_i32.abs_diff(current_output_value);
                    let delta_center = (*coefficient * center_distance as f64 * delta_t) as i32;
                    if center_distance <= min_change as u32 {
                        current_output_value = 16384_i32;
                    } else {
                        current_output_value -=
                            delta_center.max(min_change) * (current_output_value - 16384).signum();
                    }
                }
                (true, true) => (),
            }

            current_output_value
        }
    };

    value.clamp(0, 32767)
}
