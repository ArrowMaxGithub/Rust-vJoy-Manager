use std::fmt::Display;

use egui::{ComboBox, RichText, Ui};
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};

use super::IDDropdown;
use super::{
    rebind_viewer::DevicesInfoMap, shift_mode_mask::ShiftModeMask, validate_value_physical_button,
};
use crate::{error::Error, input::PhysicalDevice};

///Logical rebinds --> no routing to virtual device
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

impl Default for LogicalRebind {
    fn default() -> Self {
        Self::ButtonMomentaryEnableShiftMode {
            src_device: Default::default(),
            src_button: Default::default(),
            shift_mask: Default::default(),
        }
    }
}

impl Display for LogicalRebind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LogicalRebind")
    }
}

impl LogicalRebind {
    pub fn content_widget(&mut self, ui: &mut Ui, devices_info_map: &mut DevicesInfoMap) {
        ui.vertical(|ui| match self {
            LogicalRebind::ButtonMomentaryEnableShiftMode {
                src_device,
                src_button,
                shift_mask,
            } => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("From").strong());
                        ui.horizontal(|ui| {
                            ui.label("Device:");
                            devices_info_map.physical_devices_widget(ui, src_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Button:");
                            let max = devices_info_map.get_physical_limits(&src_device).0;
                            src_button.id_dropdown_widget(max, ui);
                        });
                    });

                    ui.add_space(20.0);

                    ui.vertical(|ui| {
                        ui.label(RichText::new("Effect").strong());
                        ui.horizontal(|ui| {
                            ui.label("Momentary enable:");
                            shift_mask.widget(ui);
                        });
                    });
                });
            }

            LogicalRebind::ButtonMomentaryDisableShiftMode {
                src_device,
                src_button,
                shift_mask,
            } => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(200.0);
                        ui.label(RichText::new("From").strong());
                        ui.horizontal(|ui| {
                            ui.label("Device:");
                            devices_info_map.physical_devices_widget(ui, src_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Button:");
                            let max = devices_info_map.get_physical_limits(&src_device).0;
                            src_button.id_dropdown_widget(max, ui);
                        });
                    });

                    ui.add_space(20.0);

                    ui.vertical(|ui| {
                        ui.vertical(|ui| {
                            ui.set_min_width(200.0);
                            ui.label(RichText::new("Effect").strong());
                            ui.horizontal(|ui| {
                                ui.label("Momentary disable:");
                                shift_mask.widget(ui);
                            });
                        });
                    });
                });
            }
        });
    }

    pub fn process(
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
