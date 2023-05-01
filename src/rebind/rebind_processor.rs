use indexmap::IndexMap;
use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
};
use vjoy::VJoy;

use crate::{
    device::{DeviceHandle, DeviceIdentifier},
    error::Error,
    input::input_state::InputState,
};

use super::{rebind::Rebind, shift_mode_mask::ShiftModeMask};

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    ButtonMomentaryShiftMode(u32, ShiftModeMask),
    ButtonMomentaryDisableShiftMode(u32, ShiftModeMask),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl Action {
    pub fn process(
        &mut self,
        src_device: &InputState,
        shift_mode: &mut ShiftModeMask,
    ) -> Result<(), Error> {
        match self {
            Action::ButtonMomentaryShiftMode(button_id, mode_mask) => {
                let Some(input) = src_device.buttons().nth(*button_id as usize - 1) else {
                    return Err(Error::ActionProcessingFailed(self.clone()))
                };

                if *input {
                    shift_mode.0 |= mode_mask.0;
                } else {
                    shift_mode.0 &= !mode_mask.0;
                }
            }

            Action::ButtonMomentaryDisableShiftMode(button_id, mode_mask) => {
                let Some(input) = src_device.buttons().nth(*button_id as usize - 1) else {
                    return Err(Error::ActionProcessingFailed(self.clone()))
                };

                if *input {
                    shift_mode.0 &= !mode_mask.0;
                } else {
                    shift_mode.0 |= mode_mask.0;
                }
            }
        }
        Ok(())
    }
}

pub struct RebindProcessor {
    active_shift_mode: ShiftModeMask,
    actions_map: HashMap<DeviceIdentifier, Vec<Action>>,
    rebind_map: HashMap<DeviceIdentifier, Vec<Rebind>>,
}

impl RebindProcessor {
    #[profiling::function]
    pub fn new() -> Self {
        let actions_map = HashMap::new();
        let rebind_map = HashMap::new();
        let active_shift_mode = ShiftModeMask(0b00000001);

        Self {
            active_shift_mode,
            actions_map,
            rebind_map,
        }
    }

    #[profiling::function]
    pub fn get_active_shift_mode(&self) -> ShiftModeMask {
        self.active_shift_mode.clone()
    }

    #[profiling::function]
    pub fn process(
        &mut self,
        devices: &mut IndexMap<DeviceIdentifier, (DeviceHandle, InputState)>,
        vjoy: &mut VJoy,
        time: f64,
        delta_t: f64,
        plot: bool,
    ) -> Result<(), Error> {
        //TODO: replace vdevices with &mut to actual handles inside devices

        //Get cached vjoy state for all vjoy devices
        let mut vdevices = {
            profiling::scope!("RebindProcessor::process::devices_cloned");
            vjoy.devices_cloned()
        };

        //Iterate all registered logic actions
        {
            profiling::scope!("RebindProcessor::process::iterate_actions");
            for (ident, actions) in self.actions_map.iter_mut() {
                let src_device = &devices.get(ident).unwrap().1;

                for action in actions.iter_mut() {
                    action.process(src_device, &mut self.active_shift_mode)?;
                }
            }
        }

        //Iterate all registered rebinds and output to cached vjoy state
        {
            profiling::scope!("RebindProcessor::process::iterate_rebinds");
            for (ident, rebinds) in self.rebind_map.iter_mut() {
                let src_device = &devices.get(ident).unwrap().1;

                for rebind in rebinds.iter_mut() {
                    let inv_required_mask = rebind.shift_mode_mask.0 ^ 0b11111111;
                    let rebind_is_active = self.active_shift_mode.0 | inv_required_mask;
                    let mut active = true;
                    for bit in 0..8 {
                        if rebind_is_active & (0b00000001 << bit) == 0 {
                            active = false;
                        }
                    }
                    if active {
                        rebind.process(src_device, &mut vdevices, delta_t)?;
                    }
                }
            }
        }

        //Output cached vjoy state to other programs and load back into device map
        {
            profiling::scope!("RebindProcessor::process::output");
            for vdevice in vdevices.into_iter() {
                {
                    profiling::scope!("RebindProcessor::process::output::ffi");
                    vjoy.update_device_state(&vdevice)?;
                }
                {
                    profiling::scope!("RebindProcessor::process::output::internal_update");
                    let (_ident, (_handle, state)) = devices
                        .iter_mut()
                        .find(|device| device.0 == &DeviceIdentifier::VJoy(vdevice.id()))
                        .unwrap();
                    state.update(&DeviceHandle::VJoy(vdevice), time, plot)?;
                }
            }
        }

        Ok(())
    }

    #[profiling::function]
    pub fn add_action(&mut self, from: &DeviceIdentifier, action: Action) {
        let actions = match self.actions_map.entry(from.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Vec::new()),
        };
        actions.push(action);
    }

    #[profiling::function]
    pub fn remove_action(&mut self, from: &DeviceIdentifier, action: Action) {
        let Some(actions) = self.actions_map.get_mut(from) else{
            return;
        };
        actions.retain(|act| *act != action);
    }

    #[profiling::function]
    pub fn clear_all_actions(&mut self) {
        self.actions_map.clear();
    }

    #[profiling::function]
    pub fn add_rebind(&mut self, from: &DeviceIdentifier, rebind: Rebind) {
        let rebinds = match self.rebind_map.entry(from.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Vec::new()),
        };
        rebinds.push(rebind);
    }

    #[profiling::function]
    pub fn remove_rebind(&mut self, from: &DeviceIdentifier, rebind: Rebind) {
        let Some(rebinds) = self.rebind_map.get_mut(from) else{
            return;
        };
        rebinds.retain(|reb| *reb != rebind);
    }

    #[profiling::function]
    pub fn clear_all_rebinds(&mut self) {
        self.rebind_map.clear();
    }
}
