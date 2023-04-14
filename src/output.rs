use std::slice::Iter;
use sdl2::joystick::HatState;
use vjoy::{VJoy, Device, ButtonState};
use crate::error::Error;

pub struct Output{
    pub vjoy: VJoy,
    pub devices: Vec<Device>,
}

impl Output{
    #[profiling::function]
    pub fn new() -> Result<Self, Error>{
        let mut vjoy = VJoy::from_default_dll_location()?;
        let devices = vjoy.devices_cloned();

        Ok(Self { vjoy, devices })
    }

    #[profiling::function]
    pub fn set_device(&mut self, device_index: usize, button_data: Iter<bool>, axes_data: Iter<i16>, hat_data: Iter<HatState>) -> Result<(), Error>{
        for (dst, src) in self.devices[device_index].buttons_mut().zip(button_data){
            let state = match src{
                true => ButtonState::Pressed,
                false => ButtonState::Released,
            };
            dst.set(state);
        }

        for (dst, src) in self.devices[device_index].axes_mut().zip(axes_data){
            let value = *src as i32;
            let state = 0 + (value - -32768) * (32767 - 0) / (32767 - -32768);

            dst.set(state);
        }

        let hat_type = self.devices[device_index].hat_type();
        for (dst, src) in self.devices[device_index].hats_mut().zip(hat_data){
            use vjoy::HatState::Discrete as disc;
            use vjoy::HatState::Continuous as cont;
            use vjoy::FourWayHat;

            let value = match (hat_type, src){
                // If only 4-way supported: clamp diagonals to north-south values
                (disc(_), HatState::Centered) => disc(FourWayHat::Centered),
                (disc(_), HatState::Up) => disc(FourWayHat::North),
                (disc(_), HatState::RightUp) => disc(FourWayHat::North),
                (disc(_), HatState::Right) => disc(FourWayHat::East),
                (disc(_), HatState::RightDown) => disc(FourWayHat::South),
                (disc(_), HatState::Down) => disc(FourWayHat::South),
                (disc(_), HatState::LeftDown) => disc(FourWayHat::South),
                (disc(_), HatState::Left) => disc(FourWayHat::West),
                (disc(_), HatState::LeftUp) => disc(FourWayHat::North),

                // Else pass value as 1/100°
                (cont(_), HatState::Centered) => cont(u32::MAX),
                (cont(_), HatState::Up) => cont(0 * 100),
                (cont(_), HatState::RightUp) => cont(45 * 100),
                (cont(_), HatState::Right) => cont(90 * 100),
                (cont(_), HatState::RightDown) => cont(135 * 100),
                (cont(_), HatState::Down) => cont(180 * 100),
                (cont(_), HatState::LeftDown) => cont(225 * 100),
                (cont(_), HatState::Left) => cont(270 * 100),
                (cont(_), HatState::LeftUp) => cont(315 * 100),
            };
            dst.set(value);
        }

        Ok(())
    }

    #[profiling::function]
    pub fn output_devices(&mut self) -> Result<(), Error>{
        for device in &self.devices{
            self.vjoy.update_device_state(device)?;
        }
        Ok(())
    }
}