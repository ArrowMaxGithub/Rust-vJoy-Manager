use ringbuffer::{AllocRingBuffer, RingBuffer, RingBufferExt, RingBufferWrite};
use vjoy::Axis;

/// Parameters (inverted, linearity etc.) and filter options for one input axis to single output axis rebinds
///
/// ## Examples usages
/// - Rebind 'X axis' to 'head movement left/right' with an inverted parameterized rebind
/// - Rebind 'Slider axis' to 'zoom in/out' and apply a 16-sample average filter (noisy input axis)
#[derive(Debug, PartialEq, Clone)]
pub enum AxisToAxisModifier {
    Parameterized(AxisParams),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AxisParams {
    pub deadzone_center: f32,
    pub clamp_min: f32,
    pub clamp_max: f32,
    pub invert: bool,
    pub linearity: f32,
    pub offset: f32,
    pub filter: Option<AllocRingBuffer<i32>>,
}

impl Default for AxisParams {
    fn default() -> Self {
        Self {
            deadzone_center: 0.0,
            clamp_min: 0.0,
            clamp_max: 1.0,
            invert: false,
            linearity: 1.0,
            offset: 0.0,
            filter: None,
        }
    }
}

// input range -32768..=32767
pub fn apply_axis_modifier(input: &i32, _output: &Axis, modifier: &mut AxisToAxisModifier) -> i32 {
    if let AxisToAxisModifier::Parameterized(params) = modifier {
        let input_f32 = match &mut params.filter {
            Some(filter) => {
                filter.push(*input);
                let count = filter.len() as f32;
                let sum = filter.iter().sum::<i32>() as f32;
                sum / count
            }
            None => *input as f32,
        };

        let inverted_value = if params.invert {
            input_f32 * -1.0
        } else {
            input_f32
        };

        let deadzone_center_min = -32768.0 * params.deadzone_center;
        let deadzone_center_max = 32767.0 * params.deadzone_center;
        let deadzone_clamped_value =
            if inverted_value >= deadzone_center_min && inverted_value <= deadzone_center_max {
                0.0
            } else {
                inverted_value
            };

        let clamp_min = -32768.0 + 32768.0 * params.clamp_min;
        let clamp_max = 32767.0 * params.clamp_max;
        let minmax_clamped_value = if deadzone_clamped_value <= clamp_min {
            -32768.0
        } else if deadzone_clamped_value >= clamp_max {
            32767.0
        } else {
            deadzone_clamped_value
        };

        let offset_value = minmax_clamped_value + (32767.0 * params.offset);
        let linearity_value = offset_value * params.linearity;

        return linearity_value.floor() as i32;
    } else {
        return *input;
    }
}

pub fn convert_axis_to_vjoy_range(input: i32) -> i32 {
    let low1 = -32768_i64;
    let high1 = 32767_i64;
    let low2 = 0_i64;
    let high2 = 32767_i64;
    let mapped_value = low2 + (input as i64 - low1) * (high2 - low2) / (high1 - low1);
    mapped_value.clamp(low2, high2) as i32
}
