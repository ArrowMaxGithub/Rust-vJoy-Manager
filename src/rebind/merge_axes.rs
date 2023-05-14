use egui::Ui;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, EnumVariantNames};

#[derive(
    Debug,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    AsRefStr,
    EnumIter,
    EnumString,
    EnumVariantNames,
)]
#[serde(tag = "modifier")]
pub enum MergeAxesModifier {
    Add,
}

impl Default for MergeAxesModifier {
    fn default() -> Self {
        Self::Add
    }
}

impl MergeAxesModifier {
    pub fn widget(&mut self, ui: &mut Ui) {
        ui.vertical(|_ui| match self {
            MergeAxesModifier::Add => {}
        });
    }
}

// input range -32768..=32767
pub fn apply_merge_axes_modifier(
    input_0: i32,
    input_1: i32,
    modifier: &mut MergeAxesModifier,
) -> i32 {
    match modifier {
        MergeAxesModifier::Add => input_0.saturating_add(input_1),
    }
}
