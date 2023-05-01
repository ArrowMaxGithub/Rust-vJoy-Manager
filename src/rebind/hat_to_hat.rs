use vjoy::{Hat, HatState};

/// Activation type and conditions for single input hat to single output hat rebinds
///
/// ## Examples usages
#[derive(Debug, PartialEq, Clone)]
pub enum HatToHatModifier {
    /// Hat maps directly to output hat
    Simple,
}

pub fn apply_hat_modifier(input: &i32, output: &Hat, modifier: &mut HatToHatModifier) -> i32 {
    match modifier {
        HatToHatModifier::Simple => *input,
    }
}

pub fn convert_hat_type_to_vjoy(hat_type: HatState, state: i32) -> vjoy::HatState {
    match hat_type {
        HatState::Discrete(_) => {
            if state == -1 {
                HatState::Discrete(vjoy::FourWayHat::Centered)
            } else if state >= 315 || state < 45 {
                HatState::Discrete(vjoy::FourWayHat::North)
            } else if state >= 45 || state < 135 {
                HatState::Discrete(vjoy::FourWayHat::East)
            } else if state >= 135 || state < 225 {
                HatState::Discrete(vjoy::FourWayHat::South)
            } else if state >= 225 || state < 315 {
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
