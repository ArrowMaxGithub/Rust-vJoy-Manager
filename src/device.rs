use std::fmt::Display;
use sdl2::joystick::Joystick;
use vjoy::Device;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DeviceIdentifier{
    VJoy(u32),
    SDL2(String),
}

impl Display for DeviceIdentifier{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceIdentifier::VJoy(id) => f.write_fmt(format_args!("{}", id)),
            DeviceIdentifier::SDL2(guid) => f.write_fmt(format_args!("{}", guid)),
        }
    }
}

pub enum DeviceHandle{
    VJoy(Device),
    SDL2(Joystick),
}

impl DeviceHandle{
    pub fn name(&self) -> String{
        match self{
            DeviceHandle::VJoy(_) => "vJoy device".to_string(),
            DeviceHandle::SDL2(handle) => handle.name(),
        }
    }
}