use egui::Ui;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default, Hash)]
pub struct ShiftModeMask(pub u8);
impl Display for ShiftModeMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:08b}", self.0))
    }
}

impl ShiftModeMask {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mask = &mut self.0;
            for i in 0..8 {
                let shift = 0b10000000 >> i;
                let bit = *mask & shift;
                ui.vertical(|ui| {
                    let label = if bit > 0 { "1" } else { "0" };
                    if ui.button(label).clicked() {
                        *mask ^= shift;
                    }
                });
            }
        });
    }
}
