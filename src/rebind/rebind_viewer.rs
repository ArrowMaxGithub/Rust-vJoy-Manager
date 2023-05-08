use egui::{Align, CentralPanel, CollapsingHeader, Context, Layout, ScrollArea, Ui, Vec2};

use super::Rebind;
use crate::{input::Input, ui_data::UIData};

pub struct RebindUIWrapped<'a> {
    pub inner: &'a mut Rebind,
    pub index: usize,
    pub keep: bool,
    pub mov: isize,
}

impl<'a> RebindUIWrapped<'a> {
    pub fn show(&mut self, ui: &mut Ui, override_open: Option<bool>) {
        ui.allocate_ui_with_layout(
            Vec2 { x: 400.0, y: 200.0 },
            Layout::left_to_right(Align::TOP),
            |ui| {
                CollapsingHeader::new(&self.inner.name)
                    .id_source(self.index)
                    .open(override_open)
                    .show_background(true)
                    .show(ui, |ui| {
                        self.inner.widget(ui);
                        ui.separator();
                    });
                if ui.button("up").clicked() {
                    self.mov = -1;
                }
                if ui.button("down").clicked() {
                    self.mov = 1;
                }
                if ui.button("X").clicked() {
                    self.keep = false;
                }
                ui.add_space(5.0);
            },
        );
    }
}

#[profiling::function]
pub(crate) fn build_ui(input: &mut Input, ctx: &Context, _ui_data: &mut UIData) {
    CentralPanel::default().show(ctx, |ui| {
        let mut active_rebinds = input.get_active_rebinds().peekable();
        if active_rebinds.peek().is_none() {
            ui.label("no active rebinds");
            return;
        }

        let mut active_rebinds_ui_wrapped: Vec<RebindUIWrapped> = active_rebinds
            .enumerate()
            .map(|(index, r)| RebindUIWrapped {
                inner: r,
                index,
                keep: true,
                mov: 0,
            })
            .collect();

        ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                let mut override_open = None;
                ui.horizontal(|ui| {
                    ui.label("Active rebinds");
                    if ui.button("collapse all").clicked() {
                        override_open = Some(false);
                    } else if ui.button("maximize all").clicked() {
                        override_open = Some(true);
                    };
                });

                ui.add_space(10.0);
                ScrollArea::vertical().show(ui, |ui| {
                    for rebind in active_rebinds_ui_wrapped.iter_mut() {
                        rebind.show(ui, override_open);
                        ui.add_space(10.0);
                    }

                    ui.add_space(ui.available_height());
                });
            });
            ui.separator();
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                ui.label("Add new rebinds");
                ui.add_space(10.0);
                ui.label("CONTENT");
            });
        });

        let keep: Vec<bool> = active_rebinds_ui_wrapped.iter().map(|r| r.keep).collect();
        let index_mov: Vec<(usize, isize)> = active_rebinds_ui_wrapped
            .iter()
            .enumerate()
            .filter_map(|(index, r)| {
                if r.mov == 0 {
                    None
                } else {
                    Some((index, r.mov))
                }
            })
            .collect();

        input.remove_rebinds_from_keep(&keep);
        for (index, mov) in index_mov {
            input.move_rebind(index, mov);
        }
    });
}
