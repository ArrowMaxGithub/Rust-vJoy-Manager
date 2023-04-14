use std::collections::HashMap;
use egui::plot::PlotPoint;
use egui::plot::PlotPoints;
use indexmap::IndexMap;
use log::trace;
use log::warn;
use ringbuffer::RingBufferWrite;
use ringbuffer::{AllocRingBuffer, RingBufferExt};
use sdl2::joystick::HatState;
use sdl2::joystick::Joystick;

use crate::error::Error;

extern crate sdl2;
use sdl2::Sdl;
use sdl2::JoystickSubsystem;

pub struct InputState{
    buttons: Vec<bool>,
    axes: Vec<i16>,
    hats: Vec<HatState>,
    axes_plot_data: Vec<AllocRingBuffer<PlotPoint>>,
}

impl InputState{
    pub fn new(device: &Joystick) -> Self{
        let buttons = (0..device.num_buttons()).map(|_|bool::default()).collect();
        let axes = (0..device.num_axes()).map(|_|i16::default()).collect();
        let hat_switches = (0..device.num_hats()).map(|_|HatState::Centered).collect();
        let axes_plot_data = (0..device.num_axes()).map(|_|AllocRingBuffer::with_capacity(1024 * 2)).collect();
        Self { buttons, axes, hats: hat_switches, axes_plot_data }
    }

    pub fn buttons(&self) -> std::slice::Iter<bool>{
        self.buttons.iter()
    }

    pub fn axes(&self) -> std::slice::Iter<i16>{
        self.axes.iter()
    }

    pub fn hats(&self) -> std::slice::Iter<HatState>{
        self.hats.iter()
    }

    pub fn update(&mut self, device: &Joystick, time: f64) -> Result<(), Error>{
        for (index, button) in self.buttons.iter_mut().enumerate(){
            *button = device.button(index as u32).unwrap()
        }

        for (index, axis) in self.axes.iter_mut().enumerate(){
            *axis = device.axis(index as u32).unwrap()
        }

        for (index, hat) in self.hats.iter_mut().enumerate(){
            *hat = device.hat(index as u32).unwrap()
        }

        for (axis_index, axis) in self.axes.iter().enumerate(){
            self.axes_plot_data[axis_index].push(PlotPoint { x: time, y: *axis as f64 });
        }

        Ok(())
    }

    pub fn axes_plot_data(&self) -> Vec<PlotPoints> {
        self.axes_plot_data
            .iter()
            .map(|buffer| PlotPoints::Owned(buffer.to_vec()))
            .collect()
    }
}

pub struct Input {
    _sdl2: Sdl,
    joystick_systen: JoystickSubsystem,
    connected_devices: IndexMap<String, String>,
    active_devices: HashMap<String, (Joystick, InputState)>,
    x_bound_min: f64,
    x_bound_max: f64,
}

impl Input {
    #[profiling::function]
    pub fn new() -> Result<Self, Error> {
        let sdl2 = sdl2::init().unwrap();
        let joystick_systen = sdl2.joystick().unwrap();

        Ok(Self {
            _sdl2: sdl2,
            joystick_systen,
            connected_devices: IndexMap::new(),
            active_devices: HashMap::new(),
            x_bound_min: 0.0,
            x_bound_max: 0.0,
        })
    }

    #[profiling::function]
    pub fn update(&mut self, time: f64) -> Result<(), Error> {
        self.joystick_systen.update();

        let num_connected_devices = self.joystick_systen.num_joysticks().unwrap();
        if num_connected_devices != self.connected_devices.len() as u32{
            self.update_connected_devices();
        }

        self.poll_active_devices(time)?;        
        
        self.x_bound_max = time;
        self.x_bound_min = time - 10.0;
        Ok(())
    }

    #[profiling::function]
    pub fn active_devices(&self) -> std::collections::hash_map::Iter<String, (Joystick, InputState)> {
        self.active_devices.iter()
    }

    #[profiling::function]
    pub fn is_active_device(&self, guid: &String) -> bool{
        self.active_devices.contains_key(guid)
    }

    #[profiling::function]
    pub fn connected_devices_count(&self) -> usize {
        self.connected_devices.len()
    }

    #[profiling::function]
    pub fn connected_devices(&self) -> indexmap::map::Iter<String, String> {
        self.connected_devices.iter()
    }

    #[profiling::function]
    pub fn toggle_active_device(&mut self, new_guid: Option<String>) {
        let Some(new_guid) = new_guid else {
            return;
        };

        if let Some((device, _data)) = self.active_devices.remove(&new_guid){
            trace!("removed active device: {} | {}", new_guid, device.name());
            return;
        }

        let Some(index) = self.connected_devices.get_index_of(&new_guid) else {
            warn!("active joystick with GUID {} not found.", new_guid);
            return;
        };

        let Ok(device) = self.joystick_systen.open(index as u32) else{
            warn!("active joystick with GUID {} could not be opened at index {}.", new_guid, index);
            return;
        };

        trace!("new active device: {} | {}", new_guid, device.name());
        let input_state = InputState::new(&device);
        self.active_devices.insert(new_guid, (device, input_state));
    }

    #[profiling::function]
    pub fn get_plot_bounds(&self) -> ([f64; 2], [f64; 2]) {
        ([self.x_bound_min, i16::MIN as f64], [self.x_bound_max, i16::MAX as f64])
    }

    #[profiling::function]
    fn update_connected_devices(&mut self){
        let num_devices = self.joystick_systen.num_joysticks().unwrap();
        self.connected_devices = (0..num_devices).map(|index|{
            (self.joystick_systen.device_guid(index).unwrap().to_string(), self.joystick_systen.name_for_index(index).unwrap())
        }).collect();

        // Check if the current active devices are still available
        self.active_devices.retain(|k, _v| self.connected_devices.contains_key(k));
    }

    #[profiling::function]
    fn poll_active_devices(&mut self, time: f64) -> Result<(), Error>{
        for (_guid, (device, input_state)) in self.active_devices.iter_mut(){
            input_state.update(device, time)?;
        }

        Ok(())
    }
}
