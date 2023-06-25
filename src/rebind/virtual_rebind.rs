use egui::{RichText, Ui};
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};

use super::{
    rebind_viewer::DevicesInfoMap,
    virtual_axis_trim::{apply_virtual_axis_trim_modifier, VirtualAxisTrimModifier},
    *,
};
use crate::{error::Error, input::VirtualDevice};

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
                TableBuilder::new(ui)
                    .column(Column::exact(TABLE_COLUMN_LEFT_WIDTH))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("Source axis").strong());
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Device:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisSrcDevice", |ui| {
                                    devices_info_map.virtual_devices_widget(ui, axis_device);
                                });
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Axis:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisSrcAxis", |ui| {
                                    let max = devices_info_map.get_virtual_limits(axis_device).1;
                                    axis.id_dropdown_widget(max, ui);
                                });
                            });
                        });
                        body.row(SECTION_SPACING, |mut row| {
                            row.col(|_| {});
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("Trim").strong());
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Pos device:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisDstTrimPosDevice", |ui| {
                                    devices_info_map.virtual_devices_widget(ui, trim_pos_device);
                                });
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Pos button:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisDstTrimPosButton", |ui| {
                                    let max =
                                        devices_info_map.get_virtual_limits(trim_pos_device).0;
                                    trim_pos_button.id_dropdown_widget(max, ui);
                                });
                            });
                        });
                        body.row(SECTION_SPACING, |mut row| {
                            row.col(|_| {});
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Neg device:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisDstTrimNegDevice", |ui| {
                                    devices_info_map.virtual_devices_widget(ui, trim_neg_device);
                                });
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Neg button:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisDstTrimNegButton", |ui| {
                                    let max =
                                        devices_info_map.get_virtual_limits(trim_neg_device).0;
                                    trim_neg_button.id_dropdown_widget(max, ui);
                                });
                            });
                        });
                        body.row(SECTION_SPACING, |mut row| {
                            row.col(|_| {});
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Reset device:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisDstTrimResetDevice", |ui| {
                                    devices_info_map.virtual_devices_widget(ui, trim_reset_device);
                                });
                            });
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label("Reset button:");
                            });
                            row.col(|ui| {
                                ui.push_id("VirtualAxisDstTrimResetButton", |ui| {
                                    let max =
                                        devices_info_map.get_virtual_limits(trim_reset_device).0;
                                    trim_reset_button.id_dropdown_widget(max, ui);
                                });
                            });
                        });
                        body.row(SECTION_SPACING, |mut row| {
                            row.col(|_| {});
                        });
                        body.row(TABLE_ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.label(RichText::new("Modifier:").strong());
                            });
                            row.col(|ui| {
                                modifier.variant_dropdown_widget(ui);
                            });
                        });
                    });

                ui.add_space(SECTION_SPACING);
                modifier.widget(ui);
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
