use std::fmt::Display;

use egui::{RichText, Ui};
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};

use super::{
    rebind_viewer::DevicesInfoMap, shift_mode_mask::ShiftModeMask, validate_value_physical_button,
};
use super::{IDDropdown, TABLE_COLUMN_LEFT_WIDTH, TABLE_ROW_HEIGHT};
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
    MomentaryEnableShiftMode {
        src_device: String,
        src_button: u32,
        shift_mask: ShiftModeMask,
    },
    MomentaryDisableShiftMode {
        src_device: String,
        src_button: u32,
        shift_mask: ShiftModeMask,
    },
}

impl Default for LogicalRebind {
    fn default() -> Self {
        Self::MomentaryEnableShiftMode {
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
            LogicalRebind::MomentaryEnableShiftMode {
                src_device,
                src_button,
                shift_mask,
            } => {
                TableBuilder::new(ui)
                    .column(Column::exact(TABLE_COLUMN_LEFT_WIDTH))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("From").strong());
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Device:");
                            });
                            row.col(|ui| {
                                devices_info_map.physical_devices_widget(ui, src_device);
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Button:");
                            });
                            row.col(|ui| {
                                let max = devices_info_map.get_physical_limits(src_device).0;
                                src_button.id_dropdown_widget(max, ui);
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|_| {});
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("Effect").strong());
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Enable:");
                            });
                            row.col(|ui| {
                                shift_mask.widget(ui);
                            });
                        });
                    });
            }

            LogicalRebind::MomentaryDisableShiftMode {
                src_device,
                src_button,
                shift_mask,
            } => {
                TableBuilder::new(ui)
                    .column(Column::exact(TABLE_COLUMN_LEFT_WIDTH))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("From").strong());
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Device:");
                            });
                            row.col(|ui| {
                                devices_info_map.physical_devices_widget(ui, src_device);
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Button:");
                            });
                            row.col(|ui| {
                                let max = devices_info_map.get_physical_limits(src_device).0;
                                src_button.id_dropdown_widget(max, ui);
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|_| {});
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("Effect").strong());
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Disable:");
                            });
                            row.col(|ui| {
                                shift_mask.widget(ui);
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
            LogicalRebind::MomentaryEnableShiftMode {
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

            LogicalRebind::MomentaryDisableShiftMode {
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
