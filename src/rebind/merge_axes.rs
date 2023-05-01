#[derive(Debug, PartialEq, Clone)]
pub enum MergeAxesModifier {
    Add,
}

// input range -32768..=32767
pub fn apply_merge_axes_modifier(
    input_0: &i32,
    input_1: &i32,
    modifier: &mut MergeAxesModifier,
) -> i32 {
    match modifier {
        MergeAxesModifier::Add => input_0.saturating_add(*input_1),
    }
}
