use crate::graphics_backend::ColorTest;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use egui_file::FileDialog;
use ringbuffer::AllocRingBuffer;
use std::{collections::HashMap, path::PathBuf};

const HAT_SWITCH: [(i32, &str); 9] = [
    (0, "north"),
    (45, "north_east"),
    (90, "east"),
    (135, "south_east"),
    (180, "south"),
    (225, "south_west"),
    (270, "west"),
    (315, "north_west"),
    (-1, "center"),
];

pub struct UIData {
    pub active_tab: ActiveTab,
    pub load_file_dialog: Option<FileDialog>,
    pub save_file_dialog: Option<FileDialog>,
    pub ferris: TextureHandle,
    pub button: TextureHandle,
    pub hat_switches: HashMap<i32, TextureHandle>,
    pub should_close: bool,
    pub color_test: ColorTest,
    pub frame_s: f64,
    pub frame_s_buffer: AllocRingBuffer<Option<f64>>,
}

impl UIData {
    #[profiling::function]
    pub fn new(ctx: &Context) -> Self {
        let ferris_img = image::open("./assets/textures/ferris.png").unwrap();
        let ferris = ctx.load_texture(
            "ferris",
            ColorImage::from_rgba_unmultiplied(
                [ferris_img.width() as usize, ferris_img.height() as usize],
                ferris_img.as_bytes(),
            ),
            TextureOptions::default(),
        );

        let button_img = image::open("./assets/textures/button.png").unwrap();
        let button = ctx.load_texture(
            "button",
            ColorImage::from_rgba_unmultiplied(
                [button_img.width() as usize, button_img.height() as usize],
                button_img.as_bytes(),
            ),
            TextureOptions::default(),
        );

        let hat_switches: HashMap<i32, TextureHandle> = HAT_SWITCH
            .iter()
            .map(|(state, name)| {
                let img = image::open(format!("./assets/textures/hat_switch/{name}.png")).unwrap();
                (
                    *state,
                    ctx.load_texture(
                        *name,
                        ColorImage::from_rgba_unmultiplied(
                            [img.width() as usize, img.height() as usize],
                            img.as_bytes(),
                        ),
                        TextureOptions::default(),
                    ),
                )
            })
            .collect();

        let color_test = ColorTest::default();

        UIData {
            active_tab: ActiveTab::InputViewerRebind,
            save_file_dialog: None,
            load_file_dialog: None,
            ferris,
            button,
            hat_switches,
            should_close: false,
            color_test,
            frame_s: 0.0,
            frame_s_buffer: AllocRingBuffer::with_capacity(16),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ActiveTab {
    #[cfg(debug_assertions)]
    ColorTest,
    InputViewerRebind,
}
