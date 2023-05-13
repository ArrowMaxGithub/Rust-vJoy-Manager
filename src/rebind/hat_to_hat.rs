use egui::Ui;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};
use vjoy::{Hat, HatState};

/// Activation type and conditions for single input hat to single output hat rebinds
///
/// ## Examples usages
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
pub enum HatToHatModifier {
    /// Hat maps directly to output hat
    Simple,
}

impl Default for HatToHatModifier {
    fn default() -> Self {
        Self::Simple
    }
}

impl HatToHatModifier {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| match self {
            HatToHatModifier::Simple => {}
        });
    }
}

pub fn apply_hat_modifier(input: i32, _output: &Hat, modifier: &mut HatToHatModifier) -> i32 {
    match modifier {
        HatToHatModifier::Simple => input,
    }
}

pub fn convert_hat_type_to_vjoy(hat_type: HatState, state: i32) -> vjoy::HatState {
    match hat_type {
        HatState::Discrete(_) => {
            if state == -1 {
                HatState::Discrete(vjoy::FourWayHat::Centered)
            } else if !(45..315).contains(&state) {
                HatState::Discrete(vjoy::FourWayHat::North)
            } else if (45..135).contains(&state) {
                HatState::Discrete(vjoy::FourWayHat::East)
            } else if (135..225).contains(&state) {
                HatState::Discrete(vjoy::FourWayHat::South)
            } else if (225..315).contains(&state) {
                HatState::Discrete(vjoy::FourWayHat::West)
            } else {
                HatState::Discrete(vjoy::FourWayHat::Centered)
            }
        }
        HatState::Continuous(_) => {
            if state == -1 {
                HatState::Continuous(u32::MAX)
            } else {
                let converted_value = (state * 100) as u32;
                HatState::Continuous(converted_value)
            }
        }
    }
}
