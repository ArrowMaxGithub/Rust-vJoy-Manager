use std::path::Path;

use crate::rebind::activation_interval::ActivationIntervalParams;
use crate::rebind::button_to_button::ButtonToButtonModifier;
use crate::rebind::logical_rebind::LogicalRebind;
use crate::rebind::merge_axes::MergeAxesModifier;
use crate::rebind::reroute_rebind::RerouteRebind;
use crate::rebind::virtual_rebind::VirtualRebind;
use crate::{
    error::Error,
    rebind::{
        axis_to_axis::{AxisParams, AxisToAxisModifier},
        hat_to_hat::HatToHatModifier,
        shift_mode_mask::ShiftModeMask,
        virtual_axis_trim::{VirtualAxisTrimModifier, VirtualAxisTrimParams},
        {Rebind, RebindType},
    },
};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub name: String,
    pub default_shift_mode: ShiftModeMask,
    pub rebinds: Vec<Rebind>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "Default config".to_string(),
            default_shift_mode: Default::default(),
            rebinds: Default::default(),
        }
    }
}

impl Config {
    pub fn write_to_path(&self, path: &Path) -> Result<(), Error> {
        let ser_toml = toml::to_string_pretty(&self)?;
        info!("Successfully serialized config file");
        std::fs::write(path, ser_toml)?;

        Ok(())
    }

    pub fn read_from_path(path: &Path) -> Result<Self, Error> {
        match std::fs::read_to_string(path) {
            Ok(string) => match toml::from_str(&string) {
                Ok(config) => {
                    info!("Successfully deserialized config file");
                    Ok(config)
                }
                Err(e) => Err(Error::Deserialization { source: e }),
            },
            Err(e) => Err(Error::IO { source: e }),
        }
    }

    pub fn read_from_path_or_default(path: &Path) -> Self {
        match Self::read_from_path(path) {
            Ok(config) => config,
            Err(_) => Config::default(),
        }
    }

    pub fn debug_xbox360_config() -> Self {
        let guid = "030003f05e0400008e02000000007200".to_string();
        let mut rebinds = Vec::new();

        rebinds.push(Rebind {
            name: "Enable_Shift_0b10000000".to_string(),
            mode_mask: ShiftModeMask(0b00000000),
            rebind_type: RebindType::Logical {
                rebind: LogicalRebind::MomentaryEnableShiftMode {
                    src_device: guid.clone(),
                    src_button: 1,
                    shift_mask: ShiftModeMask(0b10000000),
                },
            },
        });

        rebinds.push(Rebind {
            name: "Disable_Shift_0b00000001".to_string(),
            mode_mask: ShiftModeMask(0b00000000),
            rebind_type: RebindType::Logical {
                rebind: LogicalRebind::MomentaryDisableShiftMode {
                    src_device: guid.clone(),
                    src_button: 1,
                    shift_mask: ShiftModeMask(0b00000001),
                },
            },
        });

        let mut buttons: Vec<Rebind> = (5..=10)
            .map(|i| Rebind {
                name: format!("Button_{}_To_{}", i, i),
                mode_mask: ShiftModeMask(0b00000000),
                rebind_type: RebindType::Reroute {
                    rebind: RerouteRebind::ButtonToButton {
                        src_device: guid.clone(),
                        src_button: i,
                        dst_device: 1,
                        dst_button: i,
                        modifier: ButtonToButtonModifier::Simple,
                    },
                },
            })
            .collect();

        buttons.push(Rebind {
            name: format!("Button_{}_To_{}", 3, 3),
            mode_mask: ShiftModeMask(0b00000000),
            rebind_type: RebindType::Reroute {
                rebind: RerouteRebind::ButtonToButton {
                    src_device: guid.clone(),
                    src_button: 3,
                    dst_device: 1,
                    dst_button: 3,
                    modifier: ButtonToButtonModifier::Toggle { last_input: false },
                },
            },
        });

        buttons.push(Rebind {
            name: format!("Button_{}_To_{}", 3, 3),
            mode_mask: ShiftModeMask(0b00000000),
            rebind_type: RebindType::Reroute {
                rebind: RerouteRebind::ButtonToButton {
                    src_device: guid.clone(),
                    src_button: 3,
                    dst_device: 1,
                    dst_button: 3,
                    modifier: ButtonToButtonModifier::ActivationIntervalSimple {
                        params: ActivationIntervalParams::default(),
                    },
                },
            },
        });

        buttons.push(Rebind {
            name: format!("Button_{}_To_{}", 3, 3),
            mode_mask: ShiftModeMask(0b00000000),
            rebind_type: RebindType::Reroute {
                rebind: RerouteRebind::ButtonToButton {
                    src_device: guid.clone(),
                    src_button: 3,
                    dst_device: 1,
                    dst_button: 3,
                    modifier: ButtonToButtonModifier::ActivationIntervalToggle {
                        params: ActivationIntervalParams::default(),
                    },
                },
            },
        });

        let mut hats: Vec<Rebind> = (1..=1)
            .map(|i| Rebind {
                name: format!("Hat_{}_To_{}", i, i),
                mode_mask: ShiftModeMask(0b00000000),
                rebind_type: RebindType::Reroute {
                    rebind: RerouteRebind::HatToHat {
                        src_device: guid.clone(),
                        src_hat: i,
                        dst_device: 1,
                        dst_hat: i,
                        modifier: HatToHatModifier::Simple,
                    },
                },
            })
            .collect();

        let axis_params = AxisParams::new(0.00, 0.00, 1.0, false, 2.0, 0.0, Some(4));

        let mut axes: Vec<Rebind> = (1..=6)
            .map(|i| Rebind {
                name: format!("Axis_{}_To_{}", i, i),
                mode_mask: ShiftModeMask(0b00000001),
                rebind_type: RebindType::Reroute {
                    rebind: RerouteRebind::AxisToAxis {
                        src_device: guid.clone(),
                        src_axis: i,
                        dst_device: 1,
                        dst_axis: i,
                        modifier: AxisToAxisModifier::Parameterized {
                            params: axis_params.clone(),
                        },
                    },
                },
            })
            .collect();

        axes.push(Rebind {
            name: format!("Merge_Axes_{}_And_{}_To_{}", 1, 2, 9),
            mode_mask: ShiftModeMask(0b00000001),
            rebind_type: RebindType::Reroute {
                rebind: RerouteRebind::MergeAxes {
                    src_0_device: guid.clone(),
                    src_0_axis: 1,
                    src_1_device: guid,
                    src_1_axis: 2,
                    dst_device: 1,
                    dst_axis: 9,
                    modifier: MergeAxesModifier::Add,
                },
            },
        });

        rebinds.append(&mut buttons);
        rebinds.append(&mut hats);
        rebinds.append(&mut axes);

        let virtual_axis_1_trim = Rebind {
            name: "Virtual_Axis_1_Button_Trim".to_owned(),
            mode_mask: ShiftModeMask(0b00000000),
            rebind_type: RebindType::Virtual {
                rebind: VirtualRebind::VirtualAxisApplyButtonTrim {
                    axis_device: 1,
                    axis: 1,
                    trim_neg_device: 1,
                    trim_neg_button: 5,
                    trim_pos_device: 1,
                    trim_pos_button: 6,
                    trim_reset_device: 1,
                    trim_reset_button: 2,
                    modifier: VirtualAxisTrimModifier::Click {
                        params: VirtualAxisTrimParams::new(0.50),
                    },
                },
            },
        };
        rebinds.push(virtual_axis_1_trim);

        Config {
            name: "Default Config".to_string(),
            default_shift_mode: ShiftModeMask(0b00000001),
            rebinds,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use std::path::Path;

    #[test]
    #[allow(unused_must_use)]
    fn default_config() {
        let config = Config::debug_xbox360_config();
        config
            .write_to_path(Path::new("Cfg/test_config.toml"))
            .unwrap();

        let config_readback = Config::read_from_path(Path::new("Cfg/test_config.toml")).unwrap();

        assert_eq!(config, config_readback);
    }
}
