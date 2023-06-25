use egui::{Align, CollapsingHeader, ComboBox, Layout, RichText, ScrollArea, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use indexmap::IndexMap;

use super::{
    shift_mode_mask::ShiftModeMask, Rebind, RebindType, TABLE_ROW_HEIGHT,
    TABLE_TOP_BUTTONS_WIDTH,
};
use crate::{
    input::{Input, PhysicalDevice, VirtualDevice},
    ui_data::UIData,
};

pub struct RebindUIWrapped<'a> {
    pub inner: &'a mut Rebind,
    pub index: usize,
    pub keep: bool,
    pub copy: bool,
    pub mov: isize,
}

impl<'a> RebindUIWrapped<'a> {
    pub fn widget(
        &mut self,
        ui: &mut Ui,
        override_open: Option<bool>,
        devices_name_map: &mut DevicesInfoMap,
    ) {
        ui.allocate_ui_with_layout(
            Vec2 { x: 400.0, y: 600.0 },
            Layout::left_to_right(Align::TOP),
            |ui| {
                CollapsingHeader::new(&self.inner.name)
                    .id_source(self.index)
                    .open(override_open)
                    .show_background(true)
                    .show(ui, |ui| {
                        ui.add_space(5.0);
                        self.inner.widget(ui, devices_name_map);
                        ui.separator();
                    });

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui.button("X").clicked() {
                        self.keep = false;
                    }
                    if ui.button("Clone").clicked() {
                        self.copy = true;
                    }
                    if ui.button("down").clicked() {
                        self.mov = 1;
                    }
                    if ui.button("up").clicked() {
                        self.mov = -1;
                    }
                });

                ui.add_space(5.0);
            },
        );
    }
}

pub struct DevicesInfoMap {
    pub physical_devices: IndexMap<String, DeviceInfo>,
    pub virtual_devices: IndexMap<u32, DeviceInfo>,
}

impl DevicesInfoMap {
    pub fn get_physical_name(&self, guid: &String) -> &str {
        let Some(found) = self.physical_devices.get(guid) else {
            return "Unset";
        };

        &found.name
    }

    pub fn get_virtual_name(&self, id: &u32) -> &str {
        let Some(found) = self.virtual_devices.get(id) else {
            return "Unset";
        };

        &found.name
    }

    pub fn physical_devices_widget(&self, ui: &mut Ui, selected_guid: &mut String) {
        let selected_name = self.get_physical_name(selected_guid);
        ui.horizontal(|ui| {
            ui.set_min_width(200.0);
            ComboBox::from_id_source("physical_devices_widget")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for (guid, info) in self.physical_devices.iter() {
                        ui.selectable_value(selected_guid, guid.to_owned(), &info.name);
                    }
                });
        });
    }

    pub fn virtual_devices_widget(&self, ui: &mut Ui, selected_id: &mut u32) {
        let selected_name = self.get_virtual_name(selected_id);
        ui.horizontal(|ui| {
            ui.set_min_width(200.0);
            ComboBox::from_id_source("virtual_devices_widget")
                .selected_text(selected_name)
                .show_ui(ui, |ui| {
                    for (id, info) in self.virtual_devices.iter() {
                        ui.selectable_value(selected_id, id.to_owned(), &info.name);
                    }
                });
        });
    }

    pub fn get_physical_limits(&self, guid: &String) -> (u32, u32, u32) {
        let Some(found) = self.physical_devices.get(guid) else {
            return (0, 0, 0)
        };

        (
            found.num_buttons as u32,
            found.num_axes as u32,
            found.num_hats as u32,
        )
    }

    pub fn get_virtual_limits(&self, id: &u32) -> (u32, u32, u32) {
        let Some(found) = self.virtual_devices.get(id) else {
            return (0, 0, 0)
        };

        (
            found.num_buttons as u32,
            found.num_axes as u32,
            found.num_hats as u32,
        )
    }
}

pub struct DeviceInfo {
    pub name: String,
    pub num_buttons: usize,
    pub num_axes: usize,
    pub num_hats: usize,
}

impl DeviceInfo {
    pub fn from_physical(device: &PhysicalDevice) -> Self {
        Self {
            name: device.name(),
            num_buttons: device.num_buttons(),
            num_axes: device.num_axes(),
            num_hats: device.num_hats(),
        }
    }

    pub fn from_virtual(device: &VirtualDevice) -> Self {
        Self {
            name: device.name(),
            num_buttons: device.num_buttons(),
            num_axes: device.num_axes(),
            num_hats: device.num_hats(),
        }
    }
}

#[profiling::function]
pub(crate) fn build_ui(input: &mut Input, ui: &mut Ui, _ui_data: &mut UIData) {
    ui.set_height(ui.available_height());
    let physical_devices = input.get_physical_device_info_map();
    let virtual_devices = input.get_virtual_device_info_map();
    let mut devices_name_map = DevicesInfoMap {
        physical_devices,
        virtual_devices,
    };
    let mut override_open = None;

    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
            TableBuilder::new(ui)
                .column(Column::exact(TABLE_TOP_BUTTONS_WIDTH))
                .column(Column::exact(TABLE_TOP_BUTTONS_WIDTH))
                .column(Column::exact(TABLE_TOP_BUTTONS_WIDTH))
                .body(|mut body| {
                    body.row(TABLE_ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            ui.label(RichText::new("Active rebinds").strong());
                        });
                        row.col(|ui| {
                            if ui.button("Collapse all").clicked() {
                                override_open = Some(false);
                            }
                        });
                        row.col(|ui| {
                            if ui.button("Maximize all").clicked() {
                                override_open = Some(true);
                            }
                        });
                    });
                    body.row(TABLE_ROW_HEIGHT, |mut row| {
                        row.col(|ui| {
                            if ui.button("Add logical").clicked() {
                                input.add_rebind(Rebind {
                                    name: "New logical rebind".to_string(),
                                    mode_mask: ShiftModeMask::default(),
                                    rebind_type: RebindType::Logical {
                                        rebind: Default::default(),
                                    },
                                });
                            }
                        });
                        row.col(|ui| {
                            if ui.button("Add logical").clicked() {
                                input.add_rebind(Rebind {
                                    name: "New reroute rebind".to_string(),
                                    mode_mask: ShiftModeMask::default(),
                                    rebind_type: RebindType::Reroute {
                                        rebind: Default::default(),
                                    },
                                });
                            }
                        });
                        row.col(|ui| {
                            if ui.button("Add virtual").clicked() {
                                input.add_rebind(Rebind {
                                    name: "New virtual rebind".to_string(),
                                    mode_mask: ShiftModeMask::default(),
                                    rebind_type: RebindType::Virtual {
                                        rebind: Default::default(),
                                    },
                                });
                            }
                        });
                    })
                });

            ui.add_space(10.0);

            if input.get_active_rebinds().peekable().peek().is_none() {
                ui.label("no active rebinds");
                return;
            }

            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                let active_rebinds = input.get_active_rebinds().peekable();
                let mut active_rebinds_ui_wrapped: Vec<RebindUIWrapped> = active_rebinds
                    .enumerate()
                    .map(|(index, r)| RebindUIWrapped {
                        inner: r,
                        index,
                        keep: true,
                        copy: false,
                        mov: 0,
                    })
                    .collect();

                ScrollArea::vertical()
                    .always_show_scroll(true)
                    .show(ui, |ui| {
                        for rebind in active_rebinds_ui_wrapped.iter_mut() {
                            rebind.widget(ui, override_open, &mut devices_name_map);
                            ui.add_space(10.0);
                        }

                        ui.add_space(ui.available_height());
                    });

                let keep: Vec<bool> = active_rebinds_ui_wrapped.iter().map(|r| r.keep).collect();
                let copy: Vec<Rebind> = active_rebinds_ui_wrapped
                    .iter()
                    .filter_map(|r| match r.copy {
                        false => None,
                        true => Some(r.inner.clone()),
                    })
                    .collect();
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

                input.duplicate_rebinds_from_copy(copy);
            });
        });
    });
}
