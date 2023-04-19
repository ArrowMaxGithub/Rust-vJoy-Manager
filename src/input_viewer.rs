use crate::{auto_color, input::Input, ui_data::UIData};
use egui::{
    plot::{Line, Plot, PlotBounds},
    CentralPanel, Context, Image, RichText, ScrollArea, Sense, TextStyle, Widget, WidgetText,
};

#[profiling::function]
pub(crate) fn build_ui(input: &Input, ctx: &Context, ui_data: &mut UIData) {
    CentralPanel::default().show(ctx, |ui| {
        let mut plotted_devices = input.plotted_devices().peekable();

        if let None = plotted_devices.peek(){
            ui.label("no active plot - select a device from the list");
            return;
        }

        ScrollArea::vertical().show(ui, |ui| {
            for (guid, (device, data)) in plotted_devices {
                ui.label(device.name());

                ui.separator();

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(80.0);
                        ui.label("Axes");
                        for (index, axis_data) in data.axes().enumerate() {
                            ui.label(
                                RichText::new(format!("Axis {}: {}", index + 1, axis_data))
                                    .color(auto_color(index))
                                    .strong(),
                            );
                        }
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.label("Buttons");
                        if data.hats().len() > 0 {
                            ui.set_max_width(ui.available_width() - 75.0);
                        }
                        ui.horizontal_wrapped(|ui| {
                            for (index, button_state) in data.buttons().enumerate() {
                                InputButton::new((index + 1).to_string(), *button_state).ui(ui);
                            }
                        });
                    });

                    ui.separator();

                    ui.vertical(|ui| {
                        ui.vertical(|ui| {
                            for (index, hat_state) in data.hats().enumerate() {
                                let rounded = if hat_state == &-1 {
                                    -1
                                } else {
                                    (hat_state / 45) * 45
                                };

                                ui.vertical(|ui| {
                                    if let Some(texture_handle) =
                                        ui_data.hat_switches.get(&rounded)
                                    {
                                        let color = auto_color(index);
                                        ui.label(
                                            RichText::new(format!("Hat switch {index}"))
                                                .color(color),
                                        );
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

                let plot = Plot::new(format!("{guid}_axes_plot"))
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_boxed_zoom(false)
                    .height(300.0);

                plot.show(ui, |plot_ui| {
                    let plot_axes_data = data.axes_plot_data();
                    for (index, data) in plot_axes_data.into_iter().enumerate() {
                        let line = Line::new(data).width(2.0).color(auto_color(index));
                        plot_ui.line(line);
                    }
                    let (min_bound, max_bound) = input.get_plot_bounds();
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bound, max_bound));
                });
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
            egui::Vec2 { x: 25.0, y: 20.0 },
            Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        );
        let visuals = ui.style().noninteractive();
        let text_color = visuals.text_color();
        let widget_text = WidgetText::RichText(RichText::new(self.text).color(text_color));
        let galley_text = widget_text.into_galley(ui, Some(false), 25.0, TextStyle::Button);
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
