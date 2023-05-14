use crate::{
    error::Error,
    graphics_backend::Graphics,
    input::{input_viewer, Input},
    rebind::rebind_viewer,
    ui_data::{ActiveTab, UIData},
};
use egui::{
    output::OpenUrl, Align, CentralPanel, Context, FullOutput, ImageButton, Label, Layout,
    RawInput, RichText, Visuals,
};
use egui_file::FileDialog;
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

pub struct Manager {
    start: Instant,
    last_frame: Instant,
    graphics: Graphics,
    ctx: Context,
    state: State,
    ui_data: UIData,
    input: Input,
}

impl Manager {
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
                            self.ui_data.active_tab = ActiveTab::InputViewerRebind;
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
                            if let Err(e) = open_load_dialog(ui_data) {
                                error!("{e}");
                            }
                            ui.close_menu();
                        }
                        if ui.button("Save config").clicked() {
                            if let Err(e) = open_save_dialog(ui_data) {
                                error!("{e}");
                            }
                            ui.close_menu();
                        }
                        #[cfg(debug_assertions)]
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
                    if ui.button("Input viewer | Rebind").clicked() {
                        ui_data.active_tab = ActiveTab::InputViewerRebind;
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

            egui::SidePanel::left("devices")
                .default_width(100.0)
                .show(ctx, |ui| {
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

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Virtual devices:");
                            ui.label(input.virtual_devices_count().to_string());
                        });
                        ui.horizontal(|ui| {
                            ui.label("Active mode:");
                            ui.label(input.get_active_shift_mode().to_string());
                        });
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
                                    url: "https://github.com/ArrowMaxGithub/Rust-vJoy-Manager".to_string(),
                                    new_tab: true,
                                });
                            });
                        }
                        ui.label("Click to \nopen repo");
                    })
                });

            update_load_dialog(ctx, input, ui_data).unwrap();
            update_save_dialog(ctx, input, ui_data).unwrap();

            match ui_data.active_tab {
                #[cfg(debug_assertions)]
                ActiveTab::ColorTest => ui_data.color_test.build_ui(input, ctx),
                ActiveTab::InputViewerRebind => {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                            rebind_viewer::build_ui(input, ui, ui_data);
                            input_viewer::build_ui(input, ui, ui_data);
                        });
                    });
                }
            }
        })
    }
}

fn open_load_dialog(ui_data: &mut UIData) -> Result<(), Error> {
    let mut dialog = FileDialog::open_file(None).filter(Box::new(|path| match path.extension() {
        Some(os_ext) => os_ext.eq("toml"),
        None => false,
    }));
    dialog.open();
    ui_data.load_file_dialog = Some(dialog);

    Ok(())
}

fn update_load_dialog(ctx: &Context, input: &mut Input, ui_data: &mut UIData) -> Result<(), Error> {
    if let Some(dialog) = &mut ui_data.load_file_dialog {
        if dialog.show(ctx).selected() {
            if let Some(path) = dialog.path() {
                match input.load_rebinds(&path) {
                    Err(e) => error!("Failed to load rebinds from {:?}. Reason: {}", path, e),
                    Ok(_) => info!("Sucessfully loaded config from {:?}", path),
                }
            }
            ui_data.load_file_dialog = None;
        }
    }

    Ok(())
}

fn open_save_dialog(ui_data: &mut UIData) -> Result<(), Error> {
    let mut dialog = FileDialog::save_file(None).filter(Box::new(|path| match path.extension() {
        Some(os_ext) => os_ext.eq("toml"),
        None => false,
    }));
    dialog.open();
    ui_data.save_file_dialog = Some(dialog);

    Ok(())
}

fn update_save_dialog(ctx: &Context, input: &mut Input, ui_data: &mut UIData) -> Result<(), Error> {
    if let Some(dialog) = &mut ui_data.save_file_dialog {
        if dialog.show(ctx).selected() {
            if let Some(path) = dialog.path() {
                match input.save_rebinds(&path) {
                    Err(e) => error!("Failed to save rebinds to {:?}. Reason: {}", path, e),
                    Ok(_) => info!("Sucessfully saved config to {:?}", path),
                }
            }
            ui_data.save_file_dialog = None;
        }
    }

    Ok(())
}
