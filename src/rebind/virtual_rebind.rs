use egui::{ComboBox, RichText, Ui};
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};
use strum::{IntoEnumIterator, VariantNames};

use super::{
    axis_to_axis::{apply_axis_modifier, convert_axis_to_vjoy_range, AxisToAxisModifier},
    button_to_button::{apply_button_modifier, ButtonToButtonModifier},
    hat_to_hat::{apply_hat_modifier, convert_hat_type_to_vjoy, HatToHatModifier},
    merge_axes::{apply_merge_axes_modifier, MergeAxesModifier},
    rebind_viewer::DevicesInfoMap,
    two_buttons_to_axis::{apply_two_buttons_to_axis_modifier, TwoButtonsToAxisModifier},
    virtual_axis_trim::{apply_virtual_axis_trim_modifier, VirtualAxisTrimModifier},
    *,
};
use crate::{
    error::Error,
    input::{PhysicalDevice, VirtualDevice},
};

///Virtual rebinds --> modify state of virtual device(s)
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

impl Default for VirtualRebind {
    fn default() -> Self {
        Self::VirtualAxisApplyButtonTrim {
            axis_device: Default::default(),
            axis: Default::default(),
            trim_neg_device: Default::default(),
            trim_neg_button: Default::default(),
            trim_pos_device: Default::default(),
            trim_pos_button: Default::default(),
            trim_reset_device: Default::default(),
            trim_reset_button: Default::default(),
            modifier: Default::default(),
        }
    }
}

impl Display for VirtualRebind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VirtualRebind")
    }
}

impl VirtualRebind {
    pub fn content_widget(&mut self, ui: &mut Ui, devices_info_map: &mut DevicesInfoMap) {
        ui.vertical(|ui| match self {
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
                ui.push_id("Source axis", |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Source axis").strong());
                        ui.horizontal(|ui| {
                            ui.label("Device:");
                            devices_info_map.virtual_devices_widget(ui, axis_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Axis:");
                            let max = devices_info_map.get_virtual_limits(&axis_device).1;
                            axis.id_dropdown_widget(max, ui);
                        });
                    });
                });

                ui.add_space(20.0);

                ui.label(RichText::new("Trim").strong());

                ui.vertical(|ui| {
                    ui.push_id("Positive", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Positive device:");
                            devices_info_map.virtual_devices_widget(ui, trim_pos_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Positive button:");
                            let max = devices_info_map.get_virtual_limits(&trim_pos_device).0;
                            trim_pos_button.id_dropdown_widget(max, ui);
                        });
                    });

                    ui.add_space(10.0);

                    ui.push_id("Negative", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Negative device:");
                            devices_info_map.virtual_devices_widget(ui, trim_neg_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Negative button:");
                            let max = devices_info_map.get_virtual_limits(&trim_neg_device).0;
                            trim_neg_button.id_dropdown_widget(max, ui);
                        });
                    });

                    ui.add_space(10.0);

                    ui.push_id("Reset", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Reset device:");
                            devices_info_map.virtual_devices_widget(ui, trim_reset_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Reset button:");
                            let max = devices_info_map.get_virtual_limits(&trim_reset_device).0;
                            trim_reset_button.id_dropdown_widget(max, ui);
                        });
                    });
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.label("Modifier:");
                    modifier.variant_dropdown_widget(ui);
                });
            }
        });
    }

    pub fn process(
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
