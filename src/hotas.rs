use crate::{
    error::Error,
    graphics::Graphics,
    input::Input,
    input_viewer,
    output::Output,
    rebind::RebindProcessor,
    ui_data::{ActiveTab, UIData},
};
use egui::{
    output::OpenUrl, Align, CentralPanel, Context, FullOutput, ImageButton, Layout,
    RawInput, RichText, ScrollArea, Visuals, Label,
};
use egui_winit::State;
use log::info;
use ringbuffer::{RingBuffer, RingBufferExt, RingBufferWrite};
use std::time::Instant;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub struct Hotas {
    start: Instant,
    last_frame: Instant,
    graphics: Graphics,
    ctx: Context,
    state: State,
    ui_data: UIData,
    input: Input,
    rebind_processor: RebindProcessor,
    output: Output,
}

impl Hotas {
    #[profiling::function]
    pub fn new(window: &Window, event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let start = Instant::now();
        let last_frame = Instant::now();
        let graphics = Graphics::new(window)?;
        let ctx = Context::default();
        let state = State::new(&event_loop);
        let ui_data = UIData::new(&ctx);
        let input = Input::new()?;
        let rebind_processor = RebindProcessor::new();
        let output = Output::new()?;

        Ok(Self {
            start,
            last_frame,
            graphics,
            ctx,
            state,
            ui_data,
            input,
            rebind_processor,
            output,
        })
    }

    
    pub fn run(mut self, window: Window, event_loop: EventLoop<()>) -> ! {
        event_loop.run(move |new_event, _target, control_flow| {
            *control_flow = ControlFlow::Poll;
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
                if let (Some(_code), _state) = (input.virtual_keycode, input.state) {}
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

    #[profiling::function]
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
                            RichText::new(format!(
                                "{:4.0} fps",
                                1.0 / ui_data.frame_s,
                            )).color(ui.style().noninteractive().text_color().gamma_multiply(0.5))
                        );

                        let ms = Label::new(
                            RichText::new(format!(
                                "{:4.2} ms",
                                ui_data.frame_s * 1000.0,
                            )).color(ui.style().noninteractive().text_color().gamma_multiply(0.5))
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
                        "Total connected: {}",
                        input.connected_devices_count()
                    ));
                    ui.separator();
                });

                ui.separator();

                ui.vertical(|ui| {
                    for (index, (guid, (joystick, input_state))) in input.connected_devices_mut().enumerate() {
                        ui.toggle_value(&mut input_state.plot_opened, format!("{}: {} | {}", index, joystick.name(), guid));
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

            if ui_data.active_tab == ActiveTab::ColorTest {
                CentralPanel::default().show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui_data.color_test.ui(ui);
                    });
                });
            }

            if ui_data.active_tab == ActiveTab::InputViewer {
                input_viewer::build_ui(input, ctx, ui_data);
            }
        })
    }
}
