use crate::error::Error;
use sdl2::joystick::{HatState, Joystick};

pub struct InputState {
    buttons: Vec<bool>,
    axes: Vec<i32>,
    hats: Vec<i32>,
}

impl InputState {
    #[profiling::function]
    pub fn new(device: &Joystick) -> Self {
        let buttons = (0..device.num_buttons()).map(|_| bool::default()).collect();
        let axes = (0..device.num_axes()).map(|_| 0).collect();
        let hat_switches = (0..device.num_hats()).map(|_| -1).collect();

        Self {
            buttons,
            axes,
            hats: hat_switches,
        }
    }

    #[profiling::function]
    pub fn buttons(&self) -> std::slice::Iter<bool> {
        self.buttons.iter()
    }

    #[profiling::function]
    pub fn axes(&self) -> std::slice::Iter<i32> {
        self.axes.iter()
    }

    #[profiling::function]
    pub fn hats(&self) -> std::slice::Iter<i32> {
        self.hats.iter()
    }

    #[profiling::function]
    pub fn update(&mut self, device: &Joystick) -> Result<(), Error> {
        for (index, button) in self.buttons.iter_mut().enumerate() {
            *button = device.button(index as u32).unwrap()
        }

        for (index, axis) in self.axes.iter_mut().enumerate() {
            *axis = device.axis(index as u32).unwrap() as i32;
        }

        for (index, hat) in self.hats.iter_mut().enumerate() {
            *hat = match device.hat(index as u32).unwrap() {
                HatState::Centered => -1,
                HatState::Up => 0,
                HatState::Right => 90,
                HatState::Down => 180,
                HatState::Left => 270,
                HatState::RightUp => 45,
                HatState::RightDown => 135,
                HatState::LeftUp => 315,
                HatState::LeftDown => 225,
            }
        }

        Ok(())
    }
}
