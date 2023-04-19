use crate::{error::Error, input_state::InputState, device::{DeviceIdentifier, DeviceHandle}, rebind::{RebindProcessor, RebindType, Rebind}};
use indexmap::IndexMap;
use log::trace;
use sdl2::{JoystickSubsystem, Sdl};
use vjoy::VJoy;

pub struct Input {
    vjoy: VJoy,
    _sdl2: Sdl,
    joystick_systen: JoystickSubsystem,
    connected_devices: IndexMap<DeviceIdentifier, (DeviceHandle, InputState)>,
    rebind_processor: RebindProcessor,
    x_bound_min: f64,
    x_bound_max: f64,
    last_plot_time: f64,
}

impl Input {
    #[profiling::function]
    pub fn new() -> Result<Self, Error> {
        let sdl2 = sdl2::init().unwrap();
        let joystick_systen = sdl2.joystick().unwrap();
        let vjoy = VJoy::from_default_dll_location()?;
        let mut rebind_processor = RebindProcessor::new();

        rebind_processor.add_rebind(
            &DeviceIdentifier::SDL2("030003f05e0400008e02000000007200".to_string()), 
            Rebind{
                vjoy_id: 1,
                rebind_type: RebindType::All,
            },
        );

        // rebind_processor.add_rebind(
        //     &DeviceIdentifier::SDL2("03002baa1d2300000002000000000000".to_string()), 
        //     Rebind{
        //         vjoy_id: 1,
        //         rebind_type: RebindType::All,
        //     },
        // );

        Ok(Self {
            vjoy,
            _sdl2: sdl2,
            joystick_systen,
            connected_devices: IndexMap::new(),
            rebind_processor,
            x_bound_min: 0.0,
            x_bound_max: 0.0,
            last_plot_time: 0.0,
        })
    }

    
    pub fn update(&mut self, time: f64) -> Result<(), Error> {
        let num_connected_devices = self.joystick_systen.num_joysticks().unwrap();
        if num_connected_devices != self.connected_devices.len() as u32 {
            self.update_connected_devices();
        }
        let plot = time - self.last_plot_time >= 0.01;

        self.joystick_systen.update(); //update sdl2 joystick system
        self.poll_connected_physical_devices(time, plot)?; //poll sdl2 input state into cached state for all physical devices
        self.rebind_processor.process(&mut self.connected_devices, &mut self.vjoy, time, plot)?; //process rebinds and pipe cached vjoy state to vjoy output

        self.x_bound_max = time;
        self.x_bound_min = time - 10.0;
        if plot{
            self.last_plot_time = time;
        }
        Ok(())
    }

    #[profiling::function]
    pub fn connected_devices_count(&self) -> usize {
        self.connected_devices.len()
    }

    #[profiling::function]
    pub fn connected_devices(&self) -> impl Iterator<Item = (&DeviceIdentifier, &(DeviceHandle, InputState))> {
        self.connected_devices.iter()
    }

    #[profiling::function]
    pub fn connected_devices_mut(&mut self) -> impl Iterator<Item = (&DeviceIdentifier, &mut (DeviceHandle, InputState))> {
        self.connected_devices.iter_mut()
    }

    #[profiling::function]
    pub fn plotted_devices(&self) -> impl Iterator<Item = (&DeviceIdentifier, &(DeviceHandle, InputState))> {
        self.connected_devices.iter().filter(|device| device.1.1.plot_opened)
    }

    #[profiling::function]
    pub fn get_plot_bounds(&self) -> ([f64; 2], [f64; 2]) {
        (
            [self.x_bound_min, i16::MIN as f64],
            [self.x_bound_max, i16::MAX as f64],
        )
    }

    #[profiling::function]
    fn update_connected_devices(&mut self) {
        let num_devices = self.joystick_systen.num_joysticks().unwrap();
        let vjoy_devices = self.vjoy.devices_cloned();
        let mut num_vjoy = 0;

        self.connected_devices = (0..num_devices)
            .map(|index| {
                let guid = self.joystick_systen.device_guid(index).unwrap().to_string();
                let (ident, handle) = if guid == "0300f80034120000adbe000000000000"{
                    let id = vjoy_devices[num_vjoy].id();
                    let handle = vjoy_devices[num_vjoy].clone();
                    num_vjoy += 1;
                    (DeviceIdentifier::VJoy(id), DeviceHandle::VJoy(handle))
                } else {
                    let handle = self.joystick_systen.open(index).unwrap();
                    (DeviceIdentifier::SDL2(guid), DeviceHandle::SDL2(handle))
                };
                trace!("adding device: {} {ident}", handle.name());
                let input_state = InputState::new(&handle);
                (ident, (handle, input_state))
            })
            .collect();
    }

    
    fn poll_connected_physical_devices(&mut self, time: f64, plot: bool) -> Result<(), Error> {
        for (ident, (handle, state)) in self.connected_devices.iter_mut() {
            match ident{
                DeviceIdentifier::VJoy(_) => continue,
                DeviceIdentifier::SDL2(_) => state.update(handle, time, plot)?,
            }
        }

        Ok(())
    }
}
