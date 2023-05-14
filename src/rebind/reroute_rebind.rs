use egui::{RichText, Ui};
use serde::{Deserialize, Serialize};

use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};

use super::{
    axis_to_axis::{apply_axis_modifier, convert_axis_to_vjoy_range, AxisToAxisModifier},
    button_to_button::{apply_button_modifier, ButtonToButtonModifier},
    hat_to_hat::{apply_hat_modifier, convert_hat_type_to_vjoy, HatToHatModifier},
    merge_axes::{apply_merge_axes_modifier, MergeAxesModifier},
    rebind_viewer::DevicesInfoMap,
    two_buttons_to_axis::{apply_two_buttons_to_axis_modifier, TwoButtonsToAxisModifier},
    *,
};
use crate::{
    error::Error,
    input::{PhysicalDevice, VirtualDevice},
};

///Reroute rebinds --> route input from physical device(s) to virtual device
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

impl Default for RerouteRebind {
    fn default() -> Self {
        Self::ButtonToButton {
            src_device: Default::default(),
            src_button: Default::default(),
            dst_device: Default::default(),
            dst_button: Default::default(),
            modifier: Default::default(),
        }
    }
}

impl Display for RerouteRebind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RerouteRebind")
    }
}

impl RerouteRebind {
    pub fn content_widget(&mut self, ui: &mut Ui, devices_info_map: &mut DevicesInfoMap) {
        ui.vertical(|ui| match self {
            RerouteRebind::ButtonToButton {
                src_device,
                src_button,
                dst_device,
                dst_button,
                modifier,
            } => {
                ui.vertical(|ui| {
                    ui.push_id("From", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("From").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.physical_devices_widget(ui, src_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Button:");
                                let max = devices_info_map.get_physical_limits(src_device).0;
                                src_button.id_dropdown_widget(max, ui);
                            });
                        });
                    });

                    ui.add_space(SECTION_SPACING);

                    ui.push_id("To", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("To").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.virtual_devices_widget(ui, dst_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Button:");
                                let max = devices_info_map.get_virtual_limits(dst_device).0;
                                dst_button.id_dropdown_widget(max, ui);
                            });
                        });
                    });
                });

                ui.add_space(SECTION_SPACING);

                ui.push_id("modifier", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Modifier:").strong());
                        modifier.variant_dropdown_widget(ui);
                    });
                    modifier.widget(ui);
                });
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
                ui.vertical(|ui| {
                    ui.push_id("From", |ui| {
                        ui.label(RichText::new("From").strong());
                        ui.push_id("Negative", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Negative device:");
                                devices_info_map.physical_devices_widget(ui, src_neg_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Negative button:");
                                let max = devices_info_map.get_physical_limits(src_neg_device).0;
                                src_neg_button.id_dropdown_widget(max, ui);
                            });
                        });

                        ui.push_id("Positive", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Positive device:");
                                devices_info_map.physical_devices_widget(ui, src_pos_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Positive button:");
                                let max = devices_info_map.get_physical_limits(src_pos_device).0;
                                src_pos_button.id_dropdown_widget(max, ui);
                            });
                        });
                    });

                    ui.add_space(SECTION_SPACING);

                    ui.push_id("To", |ui| {
                        ui.label(RichText::new("To").strong());
                        ui.horizontal(|ui| {
                            ui.label("Device:");
                            devices_info_map.virtual_devices_widget(ui, dst_device);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Axis:");
                            let max = devices_info_map.get_virtual_limits(dst_device).0;
                            dst_axis.id_dropdown_widget(max, ui);
                        });
                    });
                });

                ui.add_space(SECTION_SPACING);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modifier:").strong());
                    modifier.variant_dropdown_widget(ui);
                });
                modifier.widget(ui);
            }

            RerouteRebind::HatToHat {
                src_device,
                src_hat,
                dst_device,
                dst_hat,
                modifier,
            } => {
                ui.vertical(|ui| {
                    ui.push_id("From", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("From").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.physical_devices_widget(ui, src_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Hat:");
                                let max = devices_info_map.get_physical_limits(src_device).2;
                                src_hat.id_dropdown_widget(max, ui);
                            });
                        });
                    });

                    ui.add_space(SECTION_SPACING);

                    ui.push_id("To", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("To").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.virtual_devices_widget(ui, dst_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Hat:");
                                let max = devices_info_map.get_virtual_limits(dst_device).2;
                                dst_hat.id_dropdown_widget(max, ui);
                            });
                        });
                    });
                });

                ui.add_space(SECTION_SPACING);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modifier:").strong());
                    modifier.variant_dropdown_widget(ui);
                });
                modifier.widget(ui);
            }

            RerouteRebind::AxisToAxis {
                src_device,
                src_axis,
                dst_device,
                dst_axis,
                modifier,
            } => {
                ui.vertical(|ui| {
                    ui.push_id("From", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("From").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.physical_devices_widget(ui, src_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Axis:");
                                let max = devices_info_map.get_physical_limits(src_device).1;
                                src_axis.id_dropdown_widget(max, ui);
                            });
                        });
                    });

                    ui.add_space(SECTION_SPACING);

                    ui.push_id("To", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("To").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.virtual_devices_widget(ui, dst_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Axis:");
                                let max = devices_info_map.get_virtual_limits(dst_device).1;
                                dst_axis.id_dropdown_widget(max, ui);
                            });
                        });
                    });
                });

                ui.add_space(SECTION_SPACING);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modifier:").strong());
                    modifier.variant_dropdown_widget(ui);
                });
                modifier.widget(ui);
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
                ui.vertical(|ui| {
                    ui.push_id("From", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("From").strong());
                            ui.push_id("First", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("First device:");
                                    devices_info_map.physical_devices_widget(ui, src_0_device);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("First axis:");
                                    let max = devices_info_map.get_physical_limits(src_0_device).1;
                                    src_0_axis.id_dropdown_widget(max, ui);
                                });
                            });
                            ui.push_id("Second", |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Second device:");
                                    devices_info_map.physical_devices_widget(ui, src_1_device);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Second axis:");
                                    let max = devices_info_map.get_physical_limits(src_1_device).1;
                                    src_1_axis.id_dropdown_widget(max, ui);
                                });
                            });
                        });
                    });

                    ui.add_space(SECTION_SPACING);

                    ui.push_id("To", |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("To").strong());
                            ui.horizontal(|ui| {
                                ui.label("Device:");
                                devices_info_map.virtual_devices_widget(ui, dst_device);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Axis:");
                                let max = devices_info_map.get_virtual_limits(dst_device).1;
                                dst_axis.id_dropdown_widget(max, ui);
                            });
                        });
                    });
                });

                ui.add_space(SECTION_SPACING);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Modifier:").strong());
                    modifier.variant_dropdown_widget(ui);
                });
                modifier.widget(ui);
            }
        });
    }

    pub fn process(
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
