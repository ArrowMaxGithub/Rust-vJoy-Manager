pub mod activation_interval;
pub mod axis_to_axis;
pub mod button_to_button;
pub mod hat_to_hat;
pub mod logical_rebind;
pub mod merge_axes;
pub mod rebind_processor;
pub mod rebind_viewer;
pub mod reroute_rebind;
pub mod shift_mode_mask;
pub mod two_buttons_to_axis;
pub mod virtual_axis_trim;
pub mod virtual_rebind;

use std::fmt::Display;

use crate::{
    error::Error,
    input::{PhysicalDevice, VirtualDevice},
};
use egui::{ComboBox, Ui};
use serde::{Deserialize, Serialize};
use vjoy::{Axis, Button, ButtonState, Hat, HatState};

use self::{
    logical_rebind::LogicalRebind, rebind_viewer::DevicesInfoMap, reroute_rebind::RerouteRebind,
    shift_mode_mask::ShiftModeMask, virtual_rebind::VirtualRebind,
};

use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};
use strum::{IntoEnumIterator, VariantNames};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Rebind {
    pub name: String,
    pub mode_mask: ShiftModeMask,

    #[serde(flatten)]
    pub rebind_type: RebindType,
}

impl Rebind {
    pub fn widget(&mut self, ui: &mut Ui, devices_name_map: &mut DevicesInfoMap) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.horizontal(|ui| {
            ui.label("Required shift mode:");
            self.mode_mask.widget(ui);
        });

        ui.add_space(10.0);

        self.rebind_type.widget(ui, devices_name_map);
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
    pub fn widget(&mut self, ui: &mut Ui, devices_name_map: &mut DevicesInfoMap) {
        match self {
            RebindType::Logical { rebind } => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Rebind type:");
                        ui.label("Logical");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rebind variant:");
                        rebind.variant_dropdown_widget(ui);
                    });
                    ui.add_space(10.0);
                    rebind.content_widget(ui, devices_name_map);
                });
            }

            RebindType::Reroute { rebind } => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Rebind type:");
                        ui.label("Reroute");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rebind variant:");
                        rebind.variant_dropdown_widget(ui);
                    });
                    ui.add_space(10.0);
                    rebind.content_widget(ui, devices_name_map);
                });
            }

            RebindType::Virtual { rebind } => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Rebind type:");
                        ui.label("Virtual");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rebind variant:");
                        rebind.variant_dropdown_widget(ui);
                    });
                    ui.add_space(10.0);
                    rebind.content_widget(ui, devices_name_map);
                });
            }
        }
    }
}

trait EnumVariantDropdown {
    fn variant_dropdown_widget(&mut self, ui: &mut Ui);
}

impl<T> EnumVariantDropdown for T
where
    T: IntoEnumIterator + AsRef<str> + VariantNames + PartialEq,
{
    fn variant_dropdown_widget(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let name_self: &str = self.as_ref();
            ComboBox::from_id_source("variant_dropdown")
                .selected_text(name_self)
                .show_ui(ui, |ui| {
                    for (i, var) in Self::iter().enumerate() {
                        ui.selectable_value(self, var, Self::VARIANTS[i]);
                    }
                });
        });
    }
}

trait IDDropdown<T> {
    fn id_dropdown_widget(&mut self, max: T, ui: &mut Ui);
}

impl IDDropdown<u32> for u32 {
    fn id_dropdown_widget(&mut self, max: u32, ui: &mut Ui) {
        ComboBox::from_id_source("id_dropdown")
            .selected_text(self.to_string())
            .show_ui(ui, |ui| {
                for i in 1..=max {
                    ui.selectable_value(self, i, i.to_string());
                }
            });
    }
}

fn validate_value_physical_button(
    physical_devices: &[PhysicalDevice],
    src_device: &String,
    src_button: &u32,
) -> Result<bool, Error> {
    if *src_button == 0 || src_device.is_empty() {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *src_hat == 0 || src_device.is_empty() {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *src_axis == 0 || src_device.is_empty() {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *src_button == 0 || *src_device == 0 || *src_device >= virtual_devices.len() as u32 {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *src_hat == 0 || *src_device == 0 || *src_device >= virtual_devices.len() as u32 {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *src_axis == 0 || *src_device == 0 || *src_device >= virtual_devices.len() as u32 {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *dst_button == 0 || *dst_device == 0 || *dst_device >= virtual_devices.len() as u32 {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *dst_hat == 0 || *dst_device == 0 || *dst_device >= virtual_devices.len() as u32 {
        return Err(Error::EmptyRebindOrInvalidID());
    }

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
    if *dst_axis == 0 || *dst_device == 0 || *dst_device >= virtual_devices.len() as u32 {
        return Err(Error::EmptyRebindOrInvalidID());
    }

    let Some(axis) = virtual_devices.iter_mut().find(|d|d.id == *dst_device)
        .and_then(|d| d.handle.axes_mut().nth(*dst_axis as usize - 1)
    ) else {
        return Err(Error::RebindValidateVirtualAxisFailed(dst_device.to_owned(), dst_axis.to_owned()))
    };
    Ok(axis)
}
