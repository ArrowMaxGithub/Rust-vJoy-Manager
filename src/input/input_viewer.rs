use crate::{auto_color, input::Input, ui_data::UIData};
use egui::{
    plot::{Line, Plot, PlotBounds},
    Image, RichText, ScrollArea, Sense, TextStyle, Ui, Widget, WidgetText,
};
use vjoy::{ButtonState, FourWayHat, HatState};

#[profiling::function]
pub(crate) fn build_ui(input: &Input, ui: &mut Ui, ui_data: &mut UIData) {
    ui.set_height(ui.available_height());

    let mut selected_physical_devices = input.selected_physical_devices().peekable();
    let mut selected_virtual_devices = input.selected_virtual_devices().peekable();

    if selected_physical_devices.peek().is_none() && selected_virtual_devices.peek().is_none() {
        ui.label("no active plot - select a device from the list");
        return;
    }

    ui.vertical(|ui| {
        ScrollArea::vertical().show(ui, |ui| {
            for device in selected_physical_devices {
                ui.label(device.name());

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(80.0);
                        for (index, axis_data) in device.input_state.axes().enumerate() {
                            ui.label(
                                RichText::new(format!("Axis {}: {}", index + 1, axis_data))
                                    .color(auto_color(index))
                                    .strong(),
                            );
                        }
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        if device.input_state.hats().len() > 0 {
                            ui.set_max_width(ui.available_width() - 75.0);
                        }
                        ui.horizontal_wrapped(|ui| {
                            for (index, button_state) in device.input_state.buttons().enumerate() {
                                InputButton::new((index + 1).to_string(), *button_state).ui(ui);
                            }
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.vertical(|ui| {
                            for (index, hat_state) in device.input_state.hats().enumerate() {
                                let rounded = if hat_state == &-1 {
                                    -1
                                } else {
                                    (hat_state / 45) * 45
                                };

                                ui.vertical(|ui| {
                                    if let Some(texture_handle) = ui_data.hat_switches.get(&rounded)
                                    {
                                        let color = auto_color(index);
                                        ui.label(
                                            RichText::new(format!("Hat {index}")).color(color),
                                        );
                                        ui.add_space(5.0);
                                        ui.add(
                                            Image::new(texture_handle.id(), [50.0, 50.0])
                                                .tint(color),
                                        );
                                    }
                                });
                            }
                        });
                    });
                });

                let plot = Plot::new(format!("{}_axes_plot", device.guid))
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .height(200.0);

                plot.show(ui, |plot_ui| {
                    let plot_axes_data = device.axes_plot_data();
                    for (index, data) in plot_axes_data.into_iter().enumerate() {
                        let line = Line::new(data).width(2.0).color(auto_color(index));
                        plot_ui.line(line);
                    }
                    let (min_bound, max_bound) = input.get_plot_bounds_physical();
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bound, max_bound));
                });
                ui.add_space(10.0);
            }

            for device in selected_virtual_devices {
                ui.label(device.name());

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(80.0);
                        for (index, axis) in device.handle.axes().enumerate() {
                            ui.label(
                                RichText::new(format!("Axis {}: {}", index + 1, axis.get()))
                                    .color(auto_color(index))
                                    .strong(),
                            );
                        }
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        if device.handle.hats().len() > 0 {
                            ui.set_max_width(ui.available_width() - 75.0);
                        }
                        ui.horizontal_wrapped(|ui| {
                            for (index, button) in device.handle.buttons().enumerate() {
                                let state = match button.get() {
                                    ButtonState::Pressed => true,
                                    ButtonState::Released => false,
                                };
                                InputButton::new((index + 1).to_string(), state).ui(ui);
                            }
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.vertical(|ui| {
                            for (index, hat) in device.handle.hats().enumerate() {
                                let rounded: i32 = match hat.get() {
                                    HatState::Continuous(value) => {
                                        if value == u32::MAX {
                                            -1
                                        } else {
                                            (value as i32 / 100 / 45) * 45
                                        }
                                    }
                                    HatState::Discrete(fourway) => match fourway {
                                        FourWayHat::Centered => -1,
                                        FourWayHat::North => 0,
                                        FourWayHat::East => 90,
                                        FourWayHat::South => 180,
                                        FourWayHat::West => 270,
                                    },
                                };

                                ui.vertical(|ui| {
                                    if let Some(texture_handle) = ui_data.hat_switches.get(&rounded)
                                    {
                                        let color = auto_color(index);
                                        ui.label(
                                            RichText::new(format!("Hat {index}")).color(color),
                                        );
                                        ui.add_space(5.0);
                                        ui.add(
                                            Image::new(texture_handle.id(), [50.0, 50.0])
                                                .tint(color),
                                        );
                                    }
                                });
                            }
                        });
                    });
                });

                let plot = Plot::new(format!("{}_axes_plot", device.name()))
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .height(200.0);

                plot.show(ui, |plot_ui| {
                    let plot_axes_data = device.axes_plot_data();
                    for (index, data) in plot_axes_data.into_iter().enumerate() {
                        let line = Line::new(data).width(2.0).color(auto_color(index));
                        plot_ui.line(line);
                    }
                    let (min_bound, max_bound) = input.get_plot_bounds_virtual();
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bound, max_bound));
                });
                ui.add_space(10.0);
            }
        });
    });
}

struct InputButton {
    text: String,
    state: bool,
}

impl InputButton {
    #[profiling::function]
    pub fn new(text: String, state: bool) -> Self {
        Self { text, state }
    }
}

impl Widget for InputButton {
    #[profiling::function]
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(
            egui::Vec2 { x: 20.0, y: 15.0 },
            Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        );
        let visuals = ui.style().noninteractive();
        let text_color = visuals.text_color();
        let widget_text =
            WidgetText::RichText(RichText::new(self.text).color(text_color).size(11.0));
        let galley_text = widget_text.into_galley(ui, Some(false), 10.0, TextStyle::Button);
        if ui.is_rect_visible(rect) {
            let fill = visuals.weak_bg_fill;
            let stroke = match self.state {
                true => visuals.fg_stroke,
                false => visuals.bg_stroke,
            };
            let rounding = visuals.rounding;
            ui.painter().rect(rect, rounding, fill, stroke);

            let padding = egui::Vec2 { x: 2.0, y: 2.0 };
            let text_pos = ui
                .layout()
                .align_size_within_rect(galley_text.size(), rect.shrink2(padding))
                .min;
            galley_text.paint_with_visuals(ui.painter(), text_pos, visuals);
        }
        response
    }
}
