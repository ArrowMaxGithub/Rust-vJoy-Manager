use crate::{
    error::Error,
    graphics_backend::Graphics,
    input::{input_viewer, Input},
    rebind::rebind_viewer,
    ui_data::{ActiveTab, UIData},
};
use egui::{
    output::OpenUrl, Align, Context, FullOutput, ImageButton, Label, Layout, RawInput, RichText,
    Visuals,
};
use egui_winit::State;
use log::{error, info};
use ringbuffer::{RingBuffer, RingBufferExt, RingBufferWrite};
use std::{
    ops::Add,
    time::{Duration, Instant},
};
use winit::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub const DEFAULT_CONFIG_LOCATION: &str = "Cfg";

pub struct Hotas {
    start: Instant,
    last_frame: Instant,
    graphics: Graphics,
    ctx: Context,
    state: State,
    ui_data: UIData,
    input: Input,
}

impl Hotas {
    #[profiling::function]
    pub fn new(window: &Window, event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let start = Instant::now();
        let last_frame = Instant::now();
        let graphics = Graphics::new(window)?;
        let ctx = Context::default();
        let state = State::new(event_loop);
        let ui_data = UIData::new(&ctx);
        let input = Input::new()?;

        Ok(Self {
            start,
            last_frame,
            graphics,
            ctx,
            state,
            ui_data,
            input,
        })
    }

    #[profiling::function]
    pub fn run(mut self, window: Window, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |new_event, _target, control_flow| {
            if window.inner_size().height == 0 || window.inner_size().height == 0 {
                *control_flow = ControlFlow::WaitUntil(
                    Instant::now().add(Duration::from_secs_f64(crate::input::INPUT_POLL_INTERVAL)),
                )
            } else {
                *control_flow = ControlFlow::Poll;
            }

            let result = match new_event {
                Event::LoopDestroyed => self.quit(),

                Event::NewEvents(_) => self.begin_new_frame(control_flow),

                Event::WindowEvent { event, .. } => {
                    self.handle_window_event(event, &window, control_flow)
                }

                Event::MainEventsCleared => self.update(&window),

                _ => Ok(()),
            };

            if let Err(err) = result {
                crate::print_error_and_exit(Box::new(err));
            }
        })
    }

    #[profiling::function]
    fn quit(&self) -> Result<(), Error> {
        self.graphics.destroy()?;
        info!("Shutdown");
        std::process::exit(0);
    }

    #[profiling::function]
    fn begin_new_frame(&mut self, control_flow: &mut ControlFlow) -> Result<(), Error> {
        profiling::finish_frame!();
        self.ui_data
            .frame_s_buffer
            .push(Some(self.last_frame.elapsed().as_secs_f64()));
        let count = self.ui_data.frame_s_buffer.len() as f64;
        self.ui_data.frame_s = self
            .ui_data
            .frame_s_buffer
            .iter()
            .fold(0.0, |acc, &v| acc + v.unwrap_or(0.0))
            / count;
        self.last_frame = Instant::now();
        if self.ui_data.should_close {
            *control_flow = ControlFlow::Exit;
        }
        Ok(())
    }

    #[profiling::function]
    fn handle_window_event(
        &mut self,
        event: WindowEvent,
        window: &Window,
        control_flow: &mut ControlFlow,
    ) -> Result<(), Error> {
        if self.state.on_event(&self.ctx, &event).consumed {
            return Ok(());
        }

        match event {
            WindowEvent::Resized(new_size) => {
                //Ignore invalid resize events during startup
                if new_size == window.inner_size() && new_size.height > 0 && new_size.width > 0 {
                    self.graphics.on_resize(new_size.into())?;
                }
            }

            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }

            WindowEvent::KeyboardInput { input, .. } => {
                if let (Some(code), state) = (input.virtual_keycode, input.state) {
                    match (code, state) {
                        (VirtualKeyCode::F1, ElementState::Pressed) => {
                            self.ui_data.active_tab = ActiveTab::InputViewer;
                        }

                        (VirtualKeyCode::F2, ElementState::Pressed) => {
                            self.ui_data.active_tab = ActiveTab::Rebind;
                        }

                        (VirtualKeyCode::F5, ElementState::Pressed) => {
                            if let Err(e) = save_config(&mut self.input) {
                                error!("{e}");
                            }
                        }

                        (VirtualKeyCode::F9, ElementState::Pressed) => {
                            if let Err(e) = load_config(&mut self.input) {
                                error!("{e}");
                            }
                        }

                        _ => (),
                    }
                }
            }

            _ => (),
        }

        Ok(())
    }

    #[profiling::function]
    fn update(&mut self, window: &Window) -> Result<(), Error> {
        self.input.update(self.start.elapsed().as_secs_f64())?;

        if window.inner_size().height == 0 || window.inner_size().height == 0 {
            return Ok(());
        }

        let raw_input = {
            profiling::scope!("egui_winit::State::take_egui_input");
            self.state.take_egui_input(window)
        };

        let full_output = Self::build_ui(&self.ctx, raw_input, &mut self.input, &mut self.ui_data);
        {
            profiling::scope!("egui_winit::State::handle_platform_output");
            self.state
                .handle_platform_output(window, &self.ctx, full_output.platform_output)
        }

        let clipped_primitives = {
            profiling::scope!("egui::Context::tessellate");
            self.ctx.tessellate(full_output.shapes)
        };

        let window_size = [
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        ];

        let ui_to_ndc = nalgebra_glm::ortho(0.0, window_size[0], 0.0, window_size[1], -1.0, 1.0);
        self.graphics
            .update(full_output.textures_delta, clipped_primitives, ui_to_ndc)?;
        Ok(())
    }

    fn build_ui(
        ctx: &Context,
        raw_input: RawInput,
        input: &mut Input,
        ui_data: &mut UIData,
    ) -> FullOutput {
        ctx.run(raw_input, |ctx| {
            egui::TopBottomPanel::top("top bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    let mut style = (*ctx.style()).clone();
                    if style.visuals.dark_mode {
                        if ui.button("☀").clicked() {
                            style.visuals = Visuals::light();
                            ctx.set_style(style);
                        }
                    } else if ui.button("🌙").clicked() {
                        style.visuals = Visuals::dark();
                        ctx.set_style(style);
                    }

                    ui.separator();
                    ui.menu_button("System", |ui| {
                        if ui.button("Load config").clicked() {
                            if let Err(e) = load_config(input) {
                                error!("{e}");
                            }
                            ui.close_menu();
                        }
                        if ui.button("Save config").clicked() {
                            if let Err(e) = save_config(input) {
                                error!("{e}");
                            }
                            ui.close_menu();
                        }
                        if ui.button("Color test").clicked() {
                            ui_data.active_tab = ActiveTab::ColorTest;
                            ui.close_menu();
                        }
                        if ui.button("Exit application").clicked() {
                            ui_data.should_close = true;
                            ui.close_menu();
                        }
                        if ui.button("Close sub menu").clicked() {
                            ui.close_menu();
                        }
                    });
                    if ui.button("Input viewer").clicked() {
                        ui_data.active_tab = ActiveTab::InputViewer;
                    }
                    if ui.button("Rebind").clicked() {
                        ui_data.active_tab = ActiveTab::Rebind;
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let fps = Label::new(
                            RichText::new(format!("{:4.0} fps", 1.0 / ui_data.frame_s,)).color(
                                ui.style().noninteractive().text_color().gamma_multiply(0.5),
                            ),
                        );

                        let ms = Label::new(
                            RichText::new(format!("{:4.2} ms", ui_data.frame_s * 1000.0,)).color(
                                ui.style().noninteractive().text_color().gamma_multiply(0.5),
                            ),
                        );

                        ui.add(fps);
                        ui.separator();
                        ui.add(ms);
                    });
                })
            });

            egui::SidePanel::left("devices").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Physical devices: {}",
                        input.physical_devices_count()
                    ));
                });

                ui.separator();

                ui.vertical(|ui| {
                    for (index, device) in input.physical_devices_mut().enumerate() {
                        let name = device.name();
                        ui.toggle_value(&mut device.selected, format!("{}: {}", index, name));
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Virtual devices: {}",
                        input.virtual_devices_count()
                    ));
                    ui.label(format!(
                        "Active shift mode: {}",
                        input.get_active_shift_mode()
                    ));
                });

                ui.separator();

                ui.vertical(|ui| {
                    for (index, device) in input.virtual_devices_mut().enumerate() {
                        let name = device.name();
                        ui.toggle_value(&mut device.selected, format!("{}: {}", index, name));
                    }
                });

                let spacing = ui.available_height() - 50.0;

                ui.add_space(spacing);

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            ImageButton::new(
                                ui_data.ferris.id(),
                                [50.0 * ui_data.ferris.aspect_ratio(), 50.0],
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        ui.ctx().output_mut(|o| {
                            o.open_url = Some(OpenUrl {
                                url: "https://github.com/ArrowMaxGithub/hotas".to_string(),
                                new_tab: true,
                            });
                        });
                    }
                    ui.label("Click to \nopen repo");
                })
            });

            if ui_data.active_tab == ActiveTab::ColorTest {}

            match ui_data.active_tab {
                ActiveTab::ColorTest => ui_data.color_test.build_ui(input, ctx),
                ActiveTab::InputViewer => input_viewer::build_ui(input, ctx, ui_data),
                ActiveTab::Rebind => rebind_viewer::build_ui(input, ctx, ui_data),
            }
        })
    }
}

fn load_config(input: &mut Input) -> Result<(), Error> {
    let load_file_path = std::env::current_dir()?
        .join(DEFAULT_CONFIG_LOCATION)
        .join("config.toml");

    match input.load_rebinds(&load_file_path) {
        Err(e) => {
            error!(
                "Failed to load rebinds from {:?}. Reason: {}",
                load_file_path, e
            )
        }
        Ok(_) => {
            info!("Sucessfully loaded config from {:?}", load_file_path)
        }
    }

    Ok(())
}

fn save_config(input: &mut Input) -> Result<(), Error> {
    let save_dir_path = std::env::current_dir()?.join(DEFAULT_CONFIG_LOCATION);
    let save_file_path = std::env::current_dir()?
        .join(DEFAULT_CONFIG_LOCATION)
        .join("config.toml");

    if !save_dir_path.exists() {
        match std::fs::create_dir(DEFAULT_CONFIG_LOCATION) {
            Err(e) => {
                error!(
                    "Failed to create save dir at {:?}. Reason: {}",
                    save_dir_path, e
                )
            }
            _ => {
                info!("Sucessfully created save dir at {:?}", save_dir_path)
            }
        }
    }
    match input.save_rebinds(&save_file_path) {
        Err(e) => {
            let source = std::error::Error::source(&e);
            error!(
                "Failed to save rebinds at {:?}. Reason: {}",
                save_file_path,
                source.unwrap_or(&e)
            )
        }
        Ok(_) => {
            info!("Sucessfully saved config at {:?}", save_file_path)
        }
    }

    Ok(())
}
