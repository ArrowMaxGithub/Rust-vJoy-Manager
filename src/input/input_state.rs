use crate::{device::DeviceHandle, error::Error};
use egui::plot::{PlotPoint, PlotPoints};
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};
use sdl2::joystick::{HatState, Joystick};
use vjoy::Device;

pub struct InputState {
    buttons: Vec<bool>,
    axes: Vec<i32>,
    hats: Vec<i32>,
    axes_plot_data: Vec<AllocRingBuffer<PlotPoint>>,
    pub plot_opened: bool,
}

impl InputState {
    #[profiling::function]
    pub fn new(handle: &DeviceHandle) -> Self {
        match handle {
            DeviceHandle::VJoy(handle) => Self::new_virtual(handle),
            DeviceHandle::SDL2(handle) => Self::new_physical(handle),
        }
    }

    #[profiling::function]
    fn new_virtual(device: &Device) -> Self {
        let buttons = device.buttons().map(|_| bool::default()).collect();
        let axes = device.axes().map(|_| 0).collect();
        let hat_switches = device.hats().map(|_| -1).collect();
        let axes_plot_data = device
            .axes()
            .map(|_| AllocRingBuffer::with_capacity(1024))
            .collect();
        Self {
            buttons,
            axes,
            hats: hat_switches,
            axes_plot_data,
            plot_opened: false,
        }
    }

    #[profiling::function]
    fn new_physical(device: &Joystick) -> Self {
        let buttons = (0..device.num_buttons()).map(|_| bool::default()).collect();
        let axes = (0..device.num_axes()).map(|_| 0).collect();
        let hat_switches = (0..device.num_hats()).map(|_| -1).collect();
        let axes_plot_data = (0..device.num_axes())
            .map(|_| AllocRingBuffer::with_capacity(1024))
            .collect();
        Self {
            buttons,
            axes,
            hats: hat_switches,
            axes_plot_data,
            plot_opened: false,
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
    pub fn update(&mut self, handle: &DeviceHandle, time: f64, plot: bool) -> Result<(), Error> {
        match handle {
            DeviceHandle::VJoy(handle) => self.update_from_virtual(handle)?,
            DeviceHandle::SDL2(handle) => self.update_from_physical(handle)?,
        };

        if !plot {
            return Ok(());
        }

        for (axis_index, axis) in self.axes.iter().enumerate() {
            self.axes_plot_data[axis_index].push(PlotPoint {
                x: time,
                y: *axis as f64,
            });
        }

        Ok(())
    }

    #[profiling::function]
    fn update_from_physical(&mut self, device: &Joystick) -> Result<(), Error> {
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

    #[profiling::function]
    fn update_from_virtual(&mut self, device: &Device) -> Result<(), Error> {
        for (button, input) in self.buttons.iter_mut().zip(device.buttons()) {
            let value = match input.get() {
                vjoy::ButtonState::Released => false,
                vjoy::ButtonState::Pressed => true,
            };
            *button = value;
        }

        for (axis, input) in self.axes.iter_mut().zip(device.axes()) {
            let value = input.get() as i64;
            let low1 = 0_i64;
            let high1 = 32767_i64;
            let low2 = -32768_i64;
            let high2 = 32767_i64;
            let mapped_value = low2 + (value - low1) * (high2 - low2) / (high1 - low1);
            *axis = mapped_value.clamp(-32768, 32767) as i32;
        }

        for (hat, input) in self.hats.iter_mut().zip(device.hats()) {
            let value = match input.get() {
                vjoy::HatState::Discrete(direction) => match direction {
                    vjoy::FourWayHat::Centered => -1,
                    vjoy::FourWayHat::North => 0,
                    vjoy::FourWayHat::East => 90,
                    vjoy::FourWayHat::South => 180,
                    vjoy::FourWayHat::West => 270,
                },
                vjoy::HatState::Continuous(angle) => {
                    if angle == u32::MAX {
                        -1
                    } else {
                        (angle as f32 / 100.0).floor() as i32
                    }
                }
            };
            *hat = value;
        }

        Ok(())
    }

    #[profiling::function]
    pub fn axes_plot_data(&self) -> Vec<PlotPoints> {
        self.axes_plot_data
            .iter()
            .map(|buffer| PlotPoints::Owned(buffer.to_vec()))
            .collect()
    }
}
