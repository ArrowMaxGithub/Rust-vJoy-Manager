pub mod activation_interval;
pub mod axis_to_axis;
pub mod button_to_button;
pub mod hat_to_hat;
pub mod merge_axes;
pub mod rebind_processor;
pub mod rebind_viewer;
pub mod shift_mode_mask;
pub mod two_buttons_to_axis;
pub mod virtual_axis_trim;

use self::{
    axis_to_axis::*, button_to_button::*, hat_to_hat::*, merge_axes::*, shift_mode_mask::*,
    two_buttons_to_axis::*, virtual_axis_trim::*,
};
use crate::{
    error::Error,
    input::{PhysicalDevice, VirtualDevice},
};
use egui::Ui;
use serde::{Deserialize, Serialize};
use vjoy::{Axis, Button, ButtonState, Hat, HatState};

///Logical rebinds --> no routing to virtual device
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant")]
pub enum LogicalRebind {
    ButtonMomentaryEnableShiftMode {
        src_device: String,
        src_button: u32,
        shift_mask: ShiftModeMask,
    },
    ButtonMomentaryDisableShiftMode {
        src_device: String,
        src_button: u32,
        shift_mask: ShiftModeMask,
    },
}

impl LogicalRebind {
    fn widget(&mut self, ui: &mut Ui) {
        ui.label("LogicalRebind CONTENT");
    }

    fn process(
        &mut self,
        physical_devices: &[PhysicalDevice],
        active_shift_mode: &mut ShiftModeMask,
    ) -> Result<(), Error> {
        match self {
            LogicalRebind::ButtonMomentaryEnableShiftMode {
                src_device,
                src_button,
                shift_mask,
            } => {
                let input =
                    validate_value_physical_button(physical_devices, src_device, src_button)?;
                if input {
                    active_shift_mode.0 |= shift_mask.0;
                } else {
                    active_shift_mode.0 &= !shift_mask.0;
                }
            }

            LogicalRebind::ButtonMomentaryDisableShiftMode {
                src_device,
                src_button,
                shift_mask,
            } => {
                let input =
                    validate_value_physical_button(physical_devices, src_device, src_button)?;
                if input {
                    active_shift_mode.0 &= !shift_mask.0;
                } else {
                    active_shift_mode.0 |= shift_mask.0;
                }
            }
        }

        Ok(())
    }
}

///Reroute rebinds --> route input from physical device(s) to virtual device
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant")]
pub enum RerouteRebind {
    ButtonToButton {
        src_device: String,
        src_button: u32,
        dst_device: u32,
        dst_button: u32,

        #[serde(flatten)]
        modifier: ButtonToButtonModifier,
    },
    TwoButtonsToAxis {
        src_neg_device: String,
        src_neg_button: u32,
        src_pos_device: String,
        src_pos_button: u32,
        dst_device: u32,
        dst_axis: u32,

        #[serde(flatten)]
        modifier: TwoButtonsToAxisModifier,
    },
    HatToHat {
        src_device: String,
        src_hat: u32,
        dst_device: u32,
        dst_hat: u32,

        #[serde(flatten)]
        modifier: HatToHatModifier,
    },
    AxisToAxis {
        src_device: String,
        src_axis: u32,
        dst_device: u32,
        dst_axis: u32,

        #[serde(flatten)]
        modifier: AxisToAxisModifier,
    },
    MergeAxes {
        src_0_device: String,
        src_0_axis: u32,
        src_1_device: String,
        src_1_axis: u32,
        dst_device: u32,
        dst_axis: u32,

        #[serde(flatten)]
        modifier: MergeAxesModifier,
    },
}

impl RerouteRebind {
    fn widget(&mut self, ui: &mut Ui) {
        ui.label("RerouteRebind CONTENT");
    }

    fn process(
        &mut self,
        physical_devices: &[PhysicalDevice],
        virtual_devices: &mut [VirtualDevice],
        time: f64,
        delta_t: f64,
    ) -> Result<(), Error> {
        match self {
            RerouteRebind::ButtonToButton {
                src_device,
                src_button,
                dst_device,
                dst_button,
                modifier,
            } => {
                let input =
                    validate_value_physical_button(physical_devices, src_device, src_button)?;
                let output =
                    validate_handle_virtual_button(virtual_devices, dst_device, dst_button)?;
                let modified_state = apply_button_modifier(input, output, modifier, time);
                output.set(modified_state);
            }

            RerouteRebind::HatToHat {
                src_device,
                src_hat,
                dst_device,
                dst_hat,
                modifier,
            } => {
                let input = validate_value_physical_hat(physical_devices, src_device, src_hat)?;
                let output = validate_handle_virtual_hat(virtual_devices, dst_device, dst_hat)?;
                let modified_state = apply_hat_modifier(input, output, modifier);
                let converted_state = convert_hat_type_to_vjoy(output.get(), modified_state);
                output.set(converted_state);
            }

            RerouteRebind::AxisToAxis {
                src_device,
                src_axis,
                dst_device,
                dst_axis,
                modifier,
            } => {
                let input = validate_value_physical_axis(physical_devices, src_device, src_axis)?;
                let output = validate_handle_virtual_axis(virtual_devices, dst_device, dst_axis)?;
                let modified_state = apply_axis_modifier(input, output, modifier);
                let converted_state = convert_axis_to_vjoy_range(modified_state);
                output.set(converted_state);
            }

            RerouteRebind::MergeAxes {
                src_0_device,
                src_0_axis,
                src_1_device,
                src_1_axis,
                dst_device,
                dst_axis,
                modifier,
            } => {
                let input_0 =
                    validate_value_physical_axis(physical_devices, src_0_device, src_0_axis)?;
                let input_1 =
                    validate_value_physical_axis(physical_devices, src_1_device, src_1_axis)?;
                let output = validate_handle_virtual_axis(virtual_devices, dst_device, dst_axis)?;
                let modified_state = apply_merge_axes_modifier(input_0, input_1, modifier);
                let converted_state = convert_axis_to_vjoy_range(modified_state);
                output.set(converted_state);
            }

            RerouteRebind::TwoButtonsToAxis {
                src_neg_device,
                src_neg_button,
                src_pos_device,
                src_pos_button,
                dst_device,
                dst_axis,
                modifier,
            } => {
                let input_neg = validate_value_physical_button(
                    physical_devices,
                    src_neg_device,
                    src_neg_button,
                )?;
                let input_pos = validate_value_physical_button(
                    physical_devices,
                    src_pos_device,
                    src_pos_button,
                )?;
                let output = validate_handle_virtual_axis(virtual_devices, dst_device, dst_axis)?;
                let modified_state = apply_two_buttons_to_axis_modifier(
                    input_neg, input_pos, output, modifier, delta_t,
                );
                output.set(modified_state);
            }
        }

        Ok(())
    }
}

///Virtual rebinds --> modify state of virtual device(s)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "variant")]
pub enum VirtualRebind {
    VirtualAxisApplyButtonTrim {
        axis_device: u32,
        axis: u32,
        trim_neg_device: u32,
        trim_neg_button: u32,
        trim_pos_device: u32,
        trim_pos_button: u32,
        trim_reset_device: u32,
        trim_reset_button: u32,

        #[serde(flatten)]
        modifier: VirtualAxisTrimModifier,
    },
}

impl VirtualRebind {
    fn widget(&mut self, ui: &mut Ui) {
        ui.label("VirtualRebind CONTENT");
    }

    fn process(
        &mut self,
        virtual_devices: &mut [VirtualDevice],
        delta_t: f64,
    ) -> Result<(), Error> {
        match self {
            VirtualRebind::VirtualAxisApplyButtonTrim {
                axis_device,
                axis,
                trim_neg_device,
                trim_neg_button,
                trim_pos_device,
                trim_pos_button,
                trim_reset_device,
                trim_reset_button,
                modifier,
            } => {
                let trim_neg = validate_value_virtual_button(
                    virtual_devices,
                    trim_neg_device,
                    trim_neg_button,
                )?;
                let trim_pos = validate_value_virtual_button(
                    virtual_devices,
                    trim_pos_device,
                    trim_pos_button,
                )?;
                let trim_reset = validate_value_virtual_button(
                    virtual_devices,
                    trim_reset_device,
                    trim_reset_button,
                )?;
                let output = validate_handle_virtual_axis(virtual_devices, axis_device, axis)?;
                let modified_state = apply_virtual_axis_trim_modifier(
                    output.get(),
                    trim_neg,
                    trim_pos,
                    trim_reset,
                    delta_t,
                    modifier,
                );
                output.set(modified_state);
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Rebind {
    pub name: String,
    pub mode_mask: ShiftModeMask,

    #[serde(flatten)]
    pub rebind_type: RebindType,
}

impl Rebind {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        self.mode_mask.widget(ui);
        self.rebind_type.widget(ui);
    }

    pub fn is_active(&self, active_shift_mode: ShiftModeMask) -> bool {
        let inv_required_mask = self.mode_mask.0 ^ 0b11111111;
        let is_active = active_shift_mode.0 | inv_required_mask;
        let mut active = true;
        for bit in 0..8 {
            if is_active & (0b00000001 << bit) == 0 {
                active = false;
            }
        }

        active
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "rebind_type")]
pub enum RebindType {
    Logical {
        #[serde(flatten)]
        rebind: LogicalRebind,
    },
    Reroute {
        #[serde(flatten)]
        rebind: RerouteRebind,
    },
    Virtual {
        #[serde(flatten)]
        rebind: VirtualRebind,
    },
}

impl RebindType {
    pub fn widget(&mut self, ui: &mut Ui) {
        match self {
            RebindType::Logical { rebind } => {
                ui.horizontal(|ui| {
                    ui.label("Rebind type:");
                    ui.label("Logical");
                });
                rebind.widget(ui);
            }

            RebindType::Reroute { rebind } => {
                ui.horizontal(|ui| {
                    ui.label("Rebind type:");
                    ui.label("Reroute");
                });
                rebind.widget(ui);
            }

            RebindType::Virtual { rebind } => {
                ui.horizontal(|ui| {
                    ui.label("Rebind type:");
                    ui.label("Virtual");
                });
                rebind.widget(ui);
            }
        }
    }
}

fn validate_value_physical_button(
    physical_devices: &[PhysicalDevice],
    src_device: &String,
    src_button: &u32,
) -> Result<bool, Error> {
    let Some(button) = physical_devices.iter().find(|d|d.guid == *src_device)
        .and_then(|d| d.input_state.buttons().nth(*src_button as usize - 1)
    ) else {
        return Err(Error::RebindValidatePhysicalButtonFailed(src_device.to_owned(), src_button.to_owned()))
    };
    Ok(*button)
}

fn validate_value_physical_hat(
    physical_devices: &[PhysicalDevice],
    src_device: &String,
    src_hat: &u32,
) -> Result<i32, Error> {
    let Some(hat) = physical_devices.iter().find(|d|d.guid == *src_device)
        .and_then(|d| d.input_state.hats().nth(*src_hat as usize - 1)
    ) else {
        return Err(Error::RebindValidatePhysicalHatFailed(src_device.to_owned(), src_hat.to_owned()))
    };
    Ok(*hat)
}

fn validate_value_physical_axis(
    physical_devices: &[PhysicalDevice],
    src_device: &String,
    src_axis: &u32,
) -> Result<i32, Error> {
    let Some(axis) = physical_devices.iter().find(|d|d.guid == *src_device)
        .and_then(|d| d.input_state.axes().nth(*src_axis as usize - 1)
    ) else {
        return Err(Error::RebindValidatePhysicalAxisFailed(src_device.to_owned(), src_axis.to_owned()))
    };
    Ok(*axis)
}

fn validate_value_virtual_button(
    virtual_devices: &[VirtualDevice],
    src_device: &u32,
    src_button: &u32,
) -> Result<bool, Error> {
    let Some(button) = virtual_devices.iter().find(|d|d.id == *src_device)
        .and_then(|d| d.handle.buttons().nth(*src_button as usize - 1).map(|b| b.get())
    ) else {
        return Err(Error::RebindValidateVirtualButtonFailed(src_device.to_owned(), src_button.to_owned()))
    };
    let button_state = match button {
        ButtonState::Released => false,
        ButtonState::Pressed => true,
    };
    Ok(button_state)
}

fn validate_value_virtual_hat(
    virtual_devices: &[VirtualDevice],
    src_device: &u32,
    src_hat: &u32,
) -> Result<HatState, Error> {
    let Some(hat) = virtual_devices.iter().find(|d|d.id == *src_device)
        .and_then(|d| d.handle.hats().nth(*src_hat as usize - 1).map(|h| h.get())
    ) else {
        return Err(Error::RebindValidateVirtualHatFailed(src_device.to_owned(), src_hat.to_owned()))
    };
    Ok(hat)
}

fn validate_value_virtual_axis(
    virtual_devices: &[VirtualDevice],
    src_device: &u32,
    src_axis: &u32,
) -> Result<i32, Error> {
    let Some(axis) = virtual_devices.iter().find(|d|d.id == *src_device)
        .and_then(|d| d.handle.axes().nth(*src_axis as usize - 1).map(|a| a.get())
    ) else {
        return Err(Error::RebindValidateVirtualAxisFailed(src_device.to_owned(), src_axis.to_owned()))
    };
    Ok(axis)
}

fn validate_handle_virtual_button<'a>(
    virtual_devices: &'a mut [VirtualDevice],
    dst_device: &u32,
    dst_button: &u32,
) -> Result<&'a mut Button, Error> {
    let Some(button) = virtual_devices.iter_mut().find(|d|d.id == *dst_device)
        .and_then(|d| d.handle.buttons_mut().nth(*dst_button as usize - 1)
    ) else {
        return Err(Error::RebindValidateVirtualButtonFailed(dst_device.to_owned(), dst_button.to_owned()))
    };
    Ok(button)
}

fn validate_handle_virtual_hat<'a>(
    virtual_devices: &'a mut [VirtualDevice],
    dst_device: &u32,
    dst_hat: &u32,
) -> Result<&'a mut Hat, Error> {
    let Some(hat) = virtual_devices.iter_mut().find(|d|d.id == *dst_device)
        .and_then(|d| d.handle.hats_mut().nth(*dst_hat as usize - 1)
    ) else {
        return Err(Error::RebindValidateVirtualHatFailed(dst_device.to_owned(), dst_hat.to_owned()))
    };
    Ok(hat)
}

fn validate_handle_virtual_axis<'a>(
    virtual_devices: &'a mut [VirtualDevice],
    dst_device: &u32,
    dst_axis: &u32,
) -> Result<&'a mut Axis, Error> {
    let Some(axis) = virtual_devices.iter_mut().find(|d|d.id == *dst_device)
        .and_then(|d| d.handle.axes_mut().nth(*dst_axis as usize - 1)
    ) else {
        return Err(Error::RebindValidateVirtualAxisFailed(dst_device.to_owned(), dst_axis.to_owned()))
    };
    Ok(axis)
}
