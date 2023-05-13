use std::path::Path;

use log::error;

use crate::{
    config::Config,
    error::Error,
    hotas::DEFAULT_CONFIG_LOCATION,
    input::{PhysicalDevice, VirtualDevice},
};

use super::{shift_mode_mask::ShiftModeMask, Rebind, RebindType};

pub struct RebindProcessor {
    config: Config,
    active_shift_mode: ShiftModeMask,
}

impl RebindProcessor {
    #[profiling::function]
    pub fn new() -> Result<Self, Error> {
        let _default_load_path = std::env::current_dir()?
            .join(DEFAULT_CONFIG_LOCATION)
            .join("config.toml");
        Ok(Self {
            // config: Config::read_from_path_or_default(&default_load_path),
            config: Config::debug_xbox360_config(),
            active_shift_mode: ShiftModeMask(0b00000000),
        })
    }

    #[profiling::function]
    pub fn add_debug_xbox360_config(&mut self) {
        self.config = Config::debug_xbox360_config();
        self.active_shift_mode = self.config.default_shift_mode;
    }

    #[profiling::function]
    pub fn save_rebinds(&self, path: &Path) -> Result<(), Error> {
        self.config.write_to_path(path)
    }

    pub fn load_rebinds(&mut self, path: &Path) -> Result<(), Error> {
        match Config::read_from_path(path) {
            Ok(config) => {
                self.config = config;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[profiling::function]
    pub fn get_active_shift_mode(&self) -> ShiftModeMask {
        self.active_shift_mode
    }

    #[profiling::function]
    pub fn get_active_rebinds(&mut self) -> std::slice::IterMut<Rebind> {
        self.config.rebinds.iter_mut()
    }

    pub fn process(
        &mut self,
        physical_devices: &mut [PhysicalDevice],
        virtual_devices: &mut [VirtualDevice],
        time: f64,
        delta_t: f64,
    ) -> Result<(), Error> {
        //Process all logical rebinds first
        for rebind in self.config.rebinds.iter_mut() {
            if !rebind.is_active(self.active_shift_mode) {
                continue;
            }

            if let RebindType::Logical { rebind } = &mut rebind.rebind_type {
                match rebind.process(physical_devices, &mut self.active_shift_mode) {
                    Ok(_) => (),
                    Err(e) => match e {
                        Error::RebindProcessingFailed(name) => error!("{name}"),
                        Error::EmptyRebindOrInvalidID() => (),
                        _ => (),
                    },
                }
            }
        }

        //Process all reroute rebinds second
        for rebind in self.config.rebinds.iter_mut() {
            if !rebind.is_active(self.active_shift_mode) {
                continue;
            }

            if let RebindType::Reroute { rebind } = &mut rebind.rebind_type {
                match rebind.process(physical_devices, virtual_devices, time, delta_t) {
                    Ok(_) => (),
                    Err(_e) => (),
                }
            }
        }

        //Process all virtual rebinds third
        for rebind in self.config.rebinds.iter_mut() {
            if !rebind.is_active(self.active_shift_mode) {
                continue;
            }

            if let RebindType::Virtual { rebind } = &mut rebind.rebind_type {
                match rebind.process(virtual_devices, delta_t) {
                    Ok(_) => (),
                    Err(_e) => (),
                }
            }
        }

        Ok(())
    }

    #[profiling::function]
    pub fn add_rebind(&mut self, rebind: Rebind) {
        self.config.rebinds.push(rebind);
    }

    #[profiling::function]
    pub fn remove_rebinds_from_keep(&mut self, keep: &[bool]) {
        let mut keep_iter = keep.iter();
        self.config
            .rebinds
            .retain(|_| *keep_iter.next().unwrap_or(&true));
    }

    #[profiling::function]
    pub fn move_rebind(&mut self, index: usize, mov: isize) {
        let swap_index = (index as isize + mov).max(0) as usize;
        if swap_index < self.config.rebinds.len() && swap_index != index {
            self.config.rebinds.swap(index, swap_index);
        }
    }

    #[profiling::function]
    pub fn clear_all_rebinds(&mut self) {
        self.config.rebinds.clear();
    }
}
