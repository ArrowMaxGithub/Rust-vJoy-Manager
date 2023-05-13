use super::activation_interval::ActivationIntervalParams;
use egui::Ui;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};
use vjoy::{Button, ButtonState};

/// Activation type and conditions for single input button to single output button rebinds
///
/// ## Examples usages
/// - Rebind 'Space' to 'jump' button without any modifier
/// - Rebind 'Shift' to 'crouch' and toggle between activation/deactivation
/// - Rebind 'F5' to two actions via two ActivationIntervalSimple rebinds:
/// 'hot-reload' if activation duration falls inside 0.0s..1.0s, 'open reload menu' if activation duration falls inside 1.0s..5.0s
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
pub enum ButtonToButtonModifier {
    /// Button maps directly to output button
    Simple,
    /// Button toggles output button
    Toggle {
        #[serde(flatten)]
        last_input: bool,
    },
    /// Output button is only activated if the current activation duration falls within this min..max interval. Output button press is sustained.
    ActivationIntervalSimple {
        #[serde(flatten)]
        params: ActivationIntervalParams,
    },
    /// ActivationInterval + Toggle
    ActivationIntervalToggle {
        #[serde(flatten)]
        params: ActivationIntervalParams,
    },
}

impl Default for ButtonToButtonModifier {
    fn default() -> Self {
        Self::Simple
    }
}

impl ButtonToButtonModifier {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| match self {
            ButtonToButtonModifier::Simple => {}

            ButtonToButtonModifier::Toggle { last_input: _ } => {}

            ButtonToButtonModifier::ActivationIntervalSimple { params } => {
                params.widget(ui);
            }

            ButtonToButtonModifier::ActivationIntervalToggle { params } => {
                params.widget(ui);
            }
        });
    }
}

pub fn apply_button_modifier(
    input: bool,
    output: &Button,
    modifier: &mut ButtonToButtonModifier,
    time: f64,
) -> ButtonState {
    match modifier {
        ButtonToButtonModifier::Simple => match input {
            true => ButtonState::Pressed,
            false => ButtonState::Released,
        },
        ButtonToButtonModifier::Toggle { last_input } => {
            let current_output_state = output.get();
            let should_toggle = input && input != *last_input;
            *last_input = input;
            if should_toggle {
                match current_output_state {
                    ButtonState::Released => ButtonState::Pressed,
                    ButtonState::Pressed => ButtonState::Released,
                }
            } else {
                current_output_state
            }
        }
        ButtonToButtonModifier::ActivationIntervalSimple { params } => {
            if params.update(input, time, true) {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            }
        }
        ButtonToButtonModifier::ActivationIntervalToggle { params } => {
            let current_output_state = output.get();
            if params.update(input, time, false) {
                match current_output_state {
                    ButtonState::Released => ButtonState::Pressed,
                    ButtonState::Pressed => ButtonState::Released,
                }
            } else {
                current_output_state
            }
        }
    }
}
