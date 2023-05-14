pub mod input_state;
pub mod input_viewer;

use std::path::Path;

use egui::plot::{PlotPoint, PlotPoints};
use indexmap::IndexMap;
use log::trace;
use ringbuffer::{AllocRingBuffer, RingBufferExt, RingBufferWrite};
use sdl2::{joystick::Joystick, JoystickSubsystem, Sdl};
use vjoy::{Device, VJoy};

use crate::{
    error::Error,
    rebind::{
        rebind_processor::RebindProcessor, rebind_viewer::DeviceInfo,
        shift_mode_mask::ShiftModeMask, Rebind,
    },
};

use self::input_state::InputState;

pub const INPUT_POLL_INTERVAL: f64 = 0.01;
pub const INPUT_PLOT_INTERVAL: f64 = 0.01;

pub struct PhysicalDevice {
    pub guid: String,
    pub handle: Joystick,
    pub input_state: InputState,
    pub axes_plot_data: Vec<AllocRingBuffer<PlotPoint>>,
    pub selected: bool,
}

impl PhysicalDevice {
    #[profiling::function]
    pub fn axes_plot_data(&self) -> Vec<PlotPoints> {
        self.axes_plot_data
            .iter()
            .map(|buffer| PlotPoints::Owned(buffer.to_vec()))
            .collect()
    }

    #[profiling::function]
    pub fn name(&self) -> String {
        self.handle.name()
    }

    #[profiling::function]
    pub fn num_buttons(&self) -> usize {
        self.input_state.num_buttons()
    }

    #[profiling::function]
    pub fn num_axes(&self) -> usize {
        self.input_state.num_axes()
    }

    #[profiling::function]
    pub fn num_hats(&self) -> usize {
        self.input_state.num_hats()
    }

    #[profiling::function]
    pub fn update(&mut self, plot: bool, time: f64) -> Result<(), Error> {
        self.input_state.update(&self.handle)?;
        if !plot {
            return Ok(());
        }

        for (axis_index, axis) in self.input_state.axes().enumerate() {
            self.axes_plot_data[axis_index].push(PlotPoint {
                x: time,
                y: *axis as f64,
            });
        }

        Ok(())
    }
}

pub struct VirtualDevice {
    pub id: u32,
    pub handle: Device,
    pub axes_plot_data: Vec<AllocRingBuffer<PlotPoint>>,
    pub selected: bool,
}

impl VirtualDevice {
    #[profiling::function]
    pub fn name(&self) -> String {
        format!("vJoy device {}", self.id)
    }

    #[profiling::function]
    pub fn num_buttons(&self) -> usize {
        self.handle.num_buttons()
    }

    #[profiling::function]
    pub fn num_axes(&self) -> usize {
        self.handle.num_axes()
    }

    #[profiling::function]
    pub fn num_hats(&self) -> usize {
        self.handle.num_hats()
    }

    #[profiling::function]
    pub fn axes_plot_data(&self) -> Vec<PlotPoints> {
        self.axes_plot_data
            .iter()
            .map(|buffer| PlotPoints::Owned(buffer.to_vec()))
            .collect()
    }

    #[profiling::function]
    pub fn update(&mut self, plot: bool, time: f64) -> Result<(), Error> {
        if !plot {
            return Ok(());
        }

        for (axis_index, axis) in self.handle.axes().enumerate() {
            self.axes_plot_data[axis_index].push(PlotPoint {
                x: time,
                y: axis.get() as f64,
            });
        }

        Ok(())
    }
}

pub struct Input {
    vjoy: VJoy,
    _sdl2: Sdl,
    joystick_systen: JoystickSubsystem,
    connected_physical_devices: Vec<PhysicalDevice>,
    active_virtual_devices: Vec<VirtualDevice>,
    rebind_processor: RebindProcessor,
    x_bound_min: f64,
    x_bound_max: f64,
    last_poll_time: f64,
    last_plot_time: f64,
}

impl Input {
    #[profiling::function]
    pub fn new() -> Result<Self, Error> {
        let sdl2 = sdl2::init()?;
        let joystick_systen = sdl2.joystick()?;
        let vjoy = VJoy::from_default_dll_location()?;
        let active_virtual_devices = Vec::new();

        let rebind_processor = RebindProcessor::new()?;

        Ok(Self {
            vjoy,
            _sdl2: sdl2,
            joystick_systen,
            connected_physical_devices: Vec::new(),
            active_virtual_devices,
            rebind_processor,
            x_bound_min: 0.0,
            x_bound_max: 0.0,
            last_poll_time: 0.0,
            last_plot_time: 0.0,
        })
    }

    pub fn update(&mut self, time: f64) -> Result<(), Error> {
        let num_connected_devices_total = self.joystick_systen.num_joysticks()?;
        if num_connected_devices_total
            != self.connected_physical_devices.len() as u32
                + self.active_virtual_devices.len() as u32
        {
            trace!("number of connected devices changed");
            self.fetch_connected_devices()?;
        }

        let delta_t = time - self.last_poll_time;
        let delta_plot = time - self.last_plot_time;

        if delta_t <= INPUT_POLL_INTERVAL {
            return Ok(());
        }

        let plot = delta_plot >= INPUT_PLOT_INTERVAL;

        //update sdl2 joystick system
        self.joystick_systen.update();

        //poll sdl2 input state into cached state for all physical devices
        self.poll_connected_physical_devices(time, plot)?;

        //process rebinds
        self.rebind_processor.process(
            &mut self.connected_physical_devices,
            &mut self.active_virtual_devices,
            time,
            delta_t,
        )?;

        //record axes data for virtual devices into plot data
        self.plot_active_virtual_devices(time, plot)?;

        //Output cached vjoy state to other programs
        {
            profiling::scope!("RebindProcessor::process::output");
            for vdevice in self.active_virtual_devices.iter() {
                self.vjoy.update_device_state(&vdevice.handle)?;
            }
        }

        self.x_bound_max = time;
        self.x_bound_min = time - 10.0;
        if plot {
            self.last_plot_time = time;
        }
        self.last_poll_time = time;
        Ok(())
    }

    #[profiling::function]
    pub fn save_rebinds(&mut self, path: &Path) -> Result<(), Error> {
        self.rebind_processor.save_rebinds(path)
    }

    #[profiling::function]
    pub fn load_rebinds(&mut self, path: &Path) -> Result<(), Error> {
        self.rebind_processor.load_rebinds(path)
    }

    #[profiling::function]
    pub fn get_physical_device_info_map(&self) -> IndexMap<String, DeviceInfo> {
        self.connected_physical_devices
            .iter()
            .map(|d| (d.guid.to_owned(), DeviceInfo::from_physical(d)))
            .collect()
    }

    #[profiling::function]
    pub fn get_virtual_device_info_map(&self) -> IndexMap<u32, DeviceInfo> {
        self.active_virtual_devices
            .iter()
            .map(|d| (d.id, DeviceInfo::from_virtual(d)))
            .collect()
    }

    #[profiling::function]
    pub fn physical_devices_count(&self) -> usize {
        self.connected_physical_devices.len()
    }

    #[profiling::function]
    pub fn physical_devices(&self) -> impl Iterator<Item = &PhysicalDevice> {
        self.connected_physical_devices.iter()
    }

    #[profiling::function]
    pub fn physical_devices_mut(&mut self) -> impl Iterator<Item = &mut PhysicalDevice> {
        self.connected_physical_devices.iter_mut()
    }

    #[profiling::function]
    pub fn selected_physical_devices(&self) -> impl Iterator<Item = &PhysicalDevice> {
        self.connected_physical_devices
            .iter()
            .filter(|device| device.selected)
    }

    #[profiling::function]
    pub fn virtual_devices_count(&self) -> usize {
        self.active_virtual_devices.len()
    }

    #[profiling::function]
    pub fn virtual_devices(&self) -> impl Iterator<Item = &VirtualDevice> {
        self.active_virtual_devices.iter()
    }

    #[profiling::function]
    pub fn virtual_devices_mut(&mut self) -> impl Iterator<Item = &mut VirtualDevice> {
        self.active_virtual_devices.iter_mut()
    }

    #[profiling::function]
    pub fn selected_virtual_devices(&self) -> impl Iterator<Item = &VirtualDevice> {
        self.active_virtual_devices
            .iter()
            .filter(|device| device.selected)
    }

    #[profiling::function]
    pub fn get_plot_bounds_physical(&self) -> ([f64; 2], [f64; 2]) {
        (
            [self.x_bound_min, i16::MIN as f64],
            [self.x_bound_max, i16::MAX as f64],
        )
    }

    #[profiling::function]
    pub fn get_plot_bounds_virtual(&self) -> ([f64; 2], [f64; 2]) {
        ([self.x_bound_min, 0.0], [self.x_bound_max, i16::MAX as f64])
    }

    #[profiling::function]
    pub fn get_active_shift_mode(&self) -> ShiftModeMask {
        self.rebind_processor.get_active_shift_mode()
    }

    #[profiling::function]
    pub fn get_active_rebinds(&mut self) -> std::slice::IterMut<Rebind> {
        self.rebind_processor.get_active_rebinds()
    }

    #[profiling::function]
    pub fn add_rebind(&mut self, rebind: Rebind) {
        self.rebind_processor.add_rebind(rebind);
    }

    #[profiling::function]
    pub fn remove_rebinds_from_keep(&mut self, keep: &[bool]) {
        self.rebind_processor.remove_rebinds_from_keep(keep);
    }

    #[profiling::function]
    pub fn duplicate_rebinds_from_copy(&mut self, copy: Vec<Rebind>) {
        self.rebind_processor.duplicate_rebinds_from_copy(copy);
    }

    #[profiling::function]
    pub fn move_rebind(&mut self, index: usize, mov: isize) {
        self.rebind_processor.move_rebind(index, mov);
    }

    #[profiling::function]
    pub fn clear_all_rebinds(&mut self) {
        self.rebind_processor.clear_all_rebinds();
    }

    fn fetch_connected_devices(&mut self) -> Result<(), Error> {
        let num_devices_total = self.joystick_systen.num_joysticks()?;
        let mut num_virtual_devices_found = 0;

        self.active_virtual_devices = self
            .vjoy
            .devices_cloned()
            .into_iter()
            .map(|vd| {
                let axes_plot_data = vd
                    .axes()
                    .map(|_| AllocRingBuffer::with_capacity(1024))
                    .collect();

                VirtualDevice {
                    id: vd.id(),
                    handle: vd,
                    axes_plot_data,
                    selected: false,
                }
            })
            .collect();

        self.connected_physical_devices = (0..num_devices_total)
            .filter_map(|index| {
                match self.joystick_systen.device_guid(index).ok() {
                    Some(guid) => {
                        let guid_str = guid.to_string();
                        // Skip vjoy guid
                        if guid_str.eq(&"0300f80034120000adbe000000000000") {
                            num_virtual_devices_found += 1;
                            None
                        } else {
                            Some((index, guid_str))
                        }
                    }
                    None => None,
                }
            })
            .filter_map(|(index, guid)| {
                let handle = self.joystick_systen.open(index);
                match handle {
                    Ok(handle) => {
                        let input_state = InputState::new(&handle);
                        let axes_plot_data = input_state
                            .axes()
                            .map(|_| AllocRingBuffer::with_capacity(1024))
                            .collect();
                        trace!("adding device: {}", handle.name());

                        Some(PhysicalDevice {
                            guid,
                            handle,
                            input_state,
                            selected: false,
                            axes_plot_data,
                        })
                    }
                    Err(_) => None,
                }
            })
            .collect();

        assert_eq!(self.active_virtual_devices.len(), num_virtual_devices_found);
        Ok(())
    }

    #[profiling::function]
    fn poll_connected_physical_devices(&mut self, time: f64, plot: bool) -> Result<(), Error> {
        for device in self.connected_physical_devices.iter_mut() {
            device.update(plot, time)?;
        }

        Ok(())
    }

    #[profiling::function]
    fn plot_active_virtual_devices(&mut self, time: f64, plot: bool) -> Result<(), Error> {
        for device in self.active_virtual_devices.iter_mut() {
            device.update(plot, time)?;
        }

        Ok(())
    }
}
