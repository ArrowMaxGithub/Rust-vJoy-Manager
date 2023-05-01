use super::{
    axis_to_axis::*, button_to_button::*, hat_to_hat::*, merge_axes::*, shift_mode_mask::*,
    two_buttons_to_axis::*,
};
use crate::{error::Error, input::input_state::InputState};
use std::fmt::Display;
use vjoy::Device;

#[derive(Debug, PartialEq, Clone)]
pub struct Rebind {
    pub shift_mode_mask: ShiftModeMask,
    pub vjoy_id: u32,
    pub rebind_type: RebindType,
}

impl Display for Rebind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?} @ device {}", self.rebind_type, self.vjoy_id))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RebindType {
    ButtonToButton(u32, u32, ButtonToButtonModifier),
    TwoButtonsToAxis(u32, u32, u32, TwoButtonsToAxisModifier),
    HatToHat(u32, u32, HatToHatModifier),
    AxisToAxis(u32, u32, AxisToAxisModifier),
    MergeAxes(u32, u32, u32, MergeAxesModifier),
}

impl Rebind {
    pub fn process(
        &mut self,
        src_device: &InputState,
        vjoy_devices: &mut Vec<Device>,
        delta_t: f64,
    ) -> Result<(), Error> {
        let Some(vjoy_device) = vjoy_devices.get_mut(self.vjoy_id as usize - 1) else {
            return Err(Error::RebindProcessingFailed(self.clone()))
        };

        match &mut self.rebind_type {
            RebindType::ButtonToButton(input_id, output_id, modifier) => {
                let Some(input) = src_device.buttons().nth(*input_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(output) = vjoy_device.buttons_mut().nth(*output_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let modified_state = apply_button_modifier(input, &output, modifier);
                output.set(modified_state);
            }

            RebindType::HatToHat(input_id, output_id, modifier) => {
                let hat_type = vjoy_device.hat_type();
                let Some(input) = src_device.hats().nth(*input_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(output) = vjoy_device.hats_mut().nth(*output_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let modified_state = apply_hat_modifier(input, &output, modifier);
                let converted_state = convert_hat_type_to_vjoy(hat_type, modified_state);
                output.set(converted_state);
            }

            RebindType::AxisToAxis(input_id, output_id, modifier) => {
                let Some(input) = src_device.axes().nth(*input_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(output) = vjoy_device.axes_mut().nth(*output_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let modified_state = apply_axis_modifier(input, &output, modifier);
                let converted_state = convert_axis_to_vjoy_range(modified_state);
                output.set(converted_state);
            }

            RebindType::MergeAxes(input_0_id, input_1_id, output_id, modifier) => {
                let Some(input_0) = src_device.axes().nth(*input_0_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(input_1) = src_device.axes().nth(*input_1_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(output) = vjoy_device.axes_mut().nth(*output_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let modified_state = apply_merge_axes_modifier(input_0, input_1, modifier);
                let converted_state = convert_axis_to_vjoy_range(modified_state);
                output.set(converted_state);
            }

            RebindType::TwoButtonsToAxis(input_neg_id, input_pos_id, output_axis_id, modifier) => {
                let Some(input_neg) = src_device.buttons().nth(*input_neg_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(input_pos) = src_device.buttons().nth(*input_pos_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let Some(output) = vjoy_device.axes_mut().nth(*output_axis_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()))
                };
                let modified_state = apply_two_buttons_to_axis_modifier(
                    input_neg, input_pos, &output, modifier, delta_t,
                );
                output.set(modified_state);
            }
        }

        Ok(())
    }
}
