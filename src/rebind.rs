use std::{collections::{hash_map::Entry, HashMap}, fmt::Display};

use indexmap::IndexMap;
use vjoy::{Device, ButtonState, HatState, VJoy};

use crate::{error::Error, input_state::InputState, device::{DeviceIdentifier, DeviceHandle}};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RebindType{
    Button(u32, u32),
    Axis(u32, u32),
    Hat(u32, u32),
    All,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Rebind{
    pub vjoy_id: u32,
    pub rebind_type: RebindType,
}

impl Display for Rebind{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("rebind: {:?} to vjoy device {}", self.rebind_type, self.vjoy_id))
    }
}

impl Rebind{
    #[profiling::function]
    pub fn process(&self, src_state: &InputState, dst_handle: &mut Device) -> Result<(), Error>{
        match self.rebind_type{
            RebindType::Button(from_id, to_id) => {
                let Some(state) = src_state.buttons().nth(from_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()));
                };
                let Some(output) = dst_handle.buttons_mut().nth(to_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()));
                };
                output.set(match state{
                    true => ButtonState::Pressed,
                    false => ButtonState::Released,
                });
            },
            RebindType::Axis(from_id, to_id) => {
                let Some(state) = src_state.axes().nth(from_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()));
                };
                let Some(output) = dst_handle.axes_mut().nth(to_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()));
                };
                //remap value 
                let value = state;
                let low1 = -32768;
                let high1 = 32767;
                let low2 = 0;
                let high2 = 32767;
                let mapped_value = low2 + (value - low1) * (high2 - low2) / (high1 - low1);
                output.set(mapped_value);
            },
            RebindType::Hat(from_id, to_id) => {
                let hat_type = dst_handle.hat_type();

                let Some(state) = src_state.hats().nth(from_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()));
                };
                let Some(output) = dst_handle.hats_mut().nth(to_id as usize - 1) else {
                    return Err(Error::RebindProcessingFailed(self.clone()));
                };
                let value = self.convert_hat_type_to_vjoy(hat_type, *state);
                output.set(value)
            },
            RebindType::All => {
                for (state, output) in src_state.buttons().zip(dst_handle.buttons_mut()){
                    output.set(match state{
                        true => ButtonState::Pressed,
                        false => ButtonState::Released,
                    });
                }
                for (state, output) in src_state.axes().zip(dst_handle.axes_mut()){
                    //remap value 
                    let value = state;
                    let low1 = -32768;
                    let high1 = 32767;
                    let low2 = 0;
                    let high2 = 32767;
                    let mapped_value = low2 + (value - low1) * (high2 - low2) / (high1 - low1);
                    output.set(mapped_value);
                }
                let hat_type = dst_handle.hat_type();
                for (state, output) in src_state.hats().zip(dst_handle.hats_mut()){
                    let value = self.convert_hat_type_to_vjoy(hat_type, *state);
                    output.set(value)
                }
            }
        }

        Ok(())
    }

    #[profiling::function]
    fn convert_hat_type_to_vjoy(&self, hat_type: HatState, state: i32) -> vjoy::HatState {
        match hat_type{
            HatState::Discrete(_) => {
                if state == -1{
                    HatState::Discrete(vjoy::FourWayHat::Centered)
                } else if state >= 315 || state < 45{
                    HatState::Discrete(vjoy::FourWayHat::North)
                } else if state >= 45 || state < 135{
                    HatState::Discrete(vjoy::FourWayHat::East)
                } else if state >= 135 || state < 225{
                    HatState::Discrete(vjoy::FourWayHat::South)
                } else if state >= 225 || state < 315{
                    HatState::Discrete(vjoy::FourWayHat::West)
                } else {
                    HatState::Discrete(vjoy::FourWayHat::Centered)
                }
            },
            HatState::Continuous(_) => {
                if state == -1{
                    HatState::Continuous(u32::MAX)
                } else {
                    let converted_value = (state * 100) as u32;
                    HatState::Continuous(converted_value)
                }
            },
        }
    }
}

pub struct RebindProcessor {
    rebind_map: HashMap<DeviceIdentifier, Vec<Rebind>>, // <(from_iany, to_vJoy_id), rebinds>
}

impl RebindProcessor {
    #[profiling::function]
    pub fn new() -> Self {
        let rebinds = HashMap::new();
        Self { rebind_map: rebinds }
    }

    #[profiling::function]
    pub fn process(&self, devices: &mut IndexMap<DeviceIdentifier, (DeviceHandle, InputState)>, vjoy: &mut VJoy, time: f64, plot: bool) -> Result<(), Error>{
        //TODO: replace vdevices with &mut to actual handles inside devices

        //Get cached vjoy state for all vjoy devices
        let mut vdevices = {
            profiling::scope!("RebindProcessor::process::devices_cloned");
             vjoy.devices_cloned()
        };

        //Iterate all registered rebinds and output to cached vjoy state
        {
            profiling::scope!("RebindProcessor::process::iterate_rebinds");
            for (ident, rebinds) in self.rebind_map.iter(){
                //src_device
                let src_state = &devices.get(ident).unwrap().1;
    
                for rebind in rebinds.iter(){
                    let mut dst_handle = &mut vdevices[(rebind.vjoy_id - 1) as usize];
                    rebind.process(src_state, &mut dst_handle)?;
                }
            }
        }

        //Output cached vjoy state to other programs and load back into device map
        {
            profiling::scope!("RebindProcessor::process::output");
            for vdevice in vdevices.into_iter(){
                {
                    profiling::scope!("RebindProcessor::process::output::ffi");
                    vjoy.update_device_state(&vdevice)?;
                }
                {
                    profiling::scope!("RebindProcessor::process::output::internal_update");
                    let (_ident, (_handle, state)) = devices.iter_mut() 
                    .find(|device|{
                        device.0 == &DeviceIdentifier::VJoy(vdevice.id())
                    })
                    .unwrap();
                    state.update(&DeviceHandle::VJoy(vdevice), time, plot)?;
                }
            }
        }   

        Ok(())
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
    pub fn clear_all(&mut self) {
        self.rebind_map.clear();
    }
}
