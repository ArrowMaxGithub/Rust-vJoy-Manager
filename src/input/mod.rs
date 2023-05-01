pub mod input_state;
pub mod input_viewer;

use indexmap::IndexMap;
use log::trace;
use ringbuffer::AllocRingBuffer;
use sdl2::{JoystickSubsystem, Sdl};
use vjoy::VJoy;

use crate::{
    device::{DeviceHandle, DeviceIdentifier},
    error::Error,
    rebind::{
        axis_to_axis::{AxisParams, AxisToAxisModifier},
        button_to_button::ButtonToButtonModifier,
        hat_to_hat::HatToHatModifier,
        merge_axes::MergeAxesModifier,
        rebind::{Rebind, RebindType},
        rebind_processor::{Action, RebindProcessor},
        shift_mode_mask::ShiftModeMask,
        two_buttons_to_axis::TwoButtonsToAxisModifier,
    },
};

use self::input_state::InputState;

pub const INPUT_POLL_INTERVAL: f64 = 0.01;
pub const INPUT_PLOT_INTERVAL: f64 = 0.01;

pub struct Input {
    vjoy: VJoy,
    _sdl2: Sdl,
    joystick_systen: JoystickSubsystem,
    connected_devices: IndexMap<DeviceIdentifier, (DeviceHandle, InputState)>,
    rebind_processor: RebindProcessor,
    x_bound_min: f64,
    x_bound_max: f64,
    last_poll_time: f64,
    last_plot_time: f64,
}

impl Input {
    pub fn debug_add_actions(&mut self, guid: String) {
        let Some(DeviceHandle::SDL2(joystick)) = self.connected_devices.iter()
            .find(|d| d.0 == &DeviceIdentifier::SDL2(guid.to_string()))
            .map(|(_ident, (handle, _state))| handle) else{
            return;
        };

        self.rebind_processor.add_action(
            &DeviceIdentifier::SDL2(joystick.guid().to_string()),
            Action::ButtonMomentaryShiftMode(1, ShiftModeMask(0b10000000)),
        );
        self.rebind_processor.add_action(
            &DeviceIdentifier::SDL2(joystick.guid().to_string()),
            Action::ButtonMomentaryDisableShiftMode(1, ShiftModeMask(0b00000001)),
        );
    }

    pub fn debug_add_rebinds(&mut self, guid: String) {
        let Some(DeviceHandle::SDL2(joystick)) = self.connected_devices.iter()
            .find(|d| d.0 == &DeviceIdentifier::SDL2(guid.to_string()))
            .map(|(_ident, (handle, _state))| handle) else{
            return;
        };

        for i in 2..=joystick.num_buttons() {
            self.rebind_processor.add_rebind(
                &DeviceIdentifier::SDL2(joystick.guid().to_string()),
                Rebind {
                    shift_mode_mask: ShiftModeMask(0b00000000),
                    vjoy_id: 1,
                    rebind_type: RebindType::ButtonToButton(i, i, ButtonToButtonModifier::Simple),
                },
            );
        }

        for i in 1..=joystick.num_hats() {
            self.rebind_processor.add_rebind(
                &DeviceIdentifier::SDL2(joystick.guid().to_string()),
                Rebind {
                    shift_mode_mask: ShiftModeMask(0b00000000),
                    vjoy_id: 1,
                    rebind_type: RebindType::HatToHat(i, i, HatToHatModifier::Simple),
                },
            );
        }

        let axis_params = AxisParams {
            deadzone_center: 0.00,
            clamp_min: 0.00,
            clamp_max: 1.0,
            invert: false,
            linearity: 1.0,
            offset: 0.0,
            filter: Some(AllocRingBuffer::with_capacity(4)),
        };
        for i in 1..=joystick.num_axes() {
            self.rebind_processor.add_rebind(
                &DeviceIdentifier::SDL2(joystick.guid().to_string()),
                Rebind {
                    shift_mode_mask: ShiftModeMask(0b00000001),
                    vjoy_id: 1,
                    rebind_type: RebindType::AxisToAxis(
                        i,
                        i,
                        AxisToAxisModifier::Parameterized(axis_params.clone()),
                    ),
                },
            );
        }

        self.rebind_processor.add_rebind(
            &DeviceIdentifier::SDL2(joystick.guid().to_string()),
            Rebind {
                shift_mode_mask: ShiftModeMask(0b10000000),
                vjoy_id: 1,
                rebind_type: RebindType::AxisToAxis(
                    1,
                    7,
                    AxisToAxisModifier::Parameterized(axis_params.clone()),
                ),
            },
        );

        self.rebind_processor.add_rebind(
            &DeviceIdentifier::VJoy(1),
            Rebind {
                shift_mode_mask: ShiftModeMask(0b00000000),
                vjoy_id: 1,
                rebind_type: RebindType::MergeAxes(1, 7, 8, MergeAxesModifier::Add),
            },
        );
    }

    #[profiling::function]
    pub fn new() -> Result<Self, Error> {
        let sdl2 = sdl2::init().unwrap();
        let joystick_systen = sdl2.joystick().unwrap();
        let vjoy = VJoy::from_default_dll_location()?;
        let rebind_processor = RebindProcessor::new();

        Ok(Self {
            vjoy,
            _sdl2: sdl2,
            joystick_systen,
            connected_devices: IndexMap::new(),
            rebind_processor,
            x_bound_min: 0.0,
            x_bound_max: 0.0,
            last_poll_time: 0.0,
            last_plot_time: 0.0,
        })
    }

    pub fn update(&mut self, time: f64) -> Result<(), Error> {
        let num_connected_devices = self.joystick_systen.num_joysticks().unwrap();
        if num_connected_devices != self.connected_devices.len() as u32 {
            self.update_connected_devices();
        }

        let delta_t = (time - self.last_poll_time) as f64;
        let delta_plot = (time - self.last_plot_time) as f64;

        if delta_t <= INPUT_POLL_INTERVAL {
            return Ok(());
        }

        let plot = delta_plot >= INPUT_PLOT_INTERVAL;

        self.joystick_systen.update(); //update sdl2 joystick system
        self.poll_connected_physical_devices(time, plot)?; //poll sdl2 input state into cached state for all physical devices
        self.rebind_processor.process(
            &mut self.connected_devices,
            &mut self.vjoy,
            time,
            delta_t,
            plot,
        )?; //process rebinds and pipe cached vjoy state to vjoy output

        self.x_bound_max = time;
        self.x_bound_min = time - 10.0;
        if plot {
            self.last_plot_time = time;
        }
        self.last_poll_time = time;
        Ok(())
    }

    #[profiling::function]
    pub fn connected_devices_count(&self) -> usize {
        self.connected_devices.len()
    }

    #[profiling::function]
    pub fn connected_devices(
        &self,
    ) -> impl Iterator<Item = (&DeviceIdentifier, &(DeviceHandle, InputState))> {
        self.connected_devices.iter()
    }

    #[profiling::function]
    pub fn connected_devices_mut(
        &mut self,
    ) -> impl Iterator<Item = (&DeviceIdentifier, &mut (DeviceHandle, InputState))> {
        self.connected_devices.iter_mut()
    }

    #[profiling::function]
    pub fn plotted_devices(
        &self,
    ) -> impl Iterator<Item = (&DeviceIdentifier, &(DeviceHandle, InputState))> {
        self.connected_devices
            .iter()
            .filter(|device| device.1 .1.plot_opened)
    }

    #[profiling::function]
    pub fn get_plot_bounds(&self) -> ([f64; 2], [f64; 2]) {
        (
            [self.x_bound_min, i16::MIN as f64],
            [self.x_bound_max, i16::MAX as f64],
        )
    }

    #[profiling::function]
    pub fn get_active_shift_mode(&self) -> ShiftModeMask {
        self.rebind_processor.get_active_shift_mode()
    }

    #[profiling::function]
    fn update_connected_devices(&mut self) {
        let num_devices = self.joystick_systen.num_joysticks().unwrap();
        let vjoy_devices = self.vjoy.devices_cloned();
        let mut num_vjoy = 0;

        self.connected_devices = (0..num_devices)
            .map(|index| {
                let guid = self.joystick_systen.device_guid(index).unwrap().to_string();
                // check for vJoy GUID
                let (ident, handle) = if guid == "0300f80034120000adbe000000000000" {
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

        self.rebind_processor.clear_all_rebinds();
        self.rebind_processor.clear_all_actions();

        self.debug_add_rebinds("030003f05e0400008e02000000007200".to_string());
        self.debug_add_actions("030003f05e0400008e02000000007200".to_string());
    }

    fn poll_connected_physical_devices(&mut self, time: f64, plot: bool) -> Result<(), Error> {
        for (ident, (handle, state)) in self.connected_devices.iter_mut() {
            match ident {
                DeviceIdentifier::VJoy(_) => continue,
                DeviceIdentifier::SDL2(_) => state.update(handle, time, plot)?,
            }
        }

        Ok(())
    }
}
