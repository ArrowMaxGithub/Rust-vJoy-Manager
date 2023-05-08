use serde::{Deserialize, Serialize};
use vjoy::Axis;

/// Parameters (inverted, linearity etc.) and filter options for one input axis to single output axis rebinds
///
/// ## Examples usages
/// - Rebind 'X axis' to 'head movement left/right' with an inverted parameterized rebind
/// - Rebind 'Slider axis' to 'zoom in/out' and apply a 16-sample average filter (noisy input axis)
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "modifier")]
pub enum AxisToAxisModifier {
    Parameterized {
        #[serde(flatten)]
        params: AxisParams,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AxisParams {
    deadzone_center: f32,
    clamp_min: f32,
    clamp_max: f32,
    invert: bool,
    linearity: f32, //Sensitivity around x=0. > 1.0 => less sensitive. < 1.0 => more sensitive. Graph: https://www.desmos.com/calculator/utdryphfaa
    offset: f32,
    avg_filter: Option<usize>,

    #[serde(skip_serializing)]
    #[serde(default)]
    avg_data: (usize, Vec<i32>),
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
            avg_filter: None,
            avg_data: (0, Vec::new()),
        }
    }
}

impl AxisParams {
    pub fn new(
        deadzone_center: f32,
        clamp_min: f32,
        clamp_max: f32,
        invert: bool,
        linearity: f32,
        offset: f32,
        avg_filter: Option<usize>,
    ) -> Self {
        Self {
            deadzone_center,
            clamp_min,
            clamp_max,
            invert,
            linearity: linearity.clamp(0.1, 10.0),
            offset,
            avg_filter,
            ..Default::default()
        }
    }
}

// input range -32768..=32767
pub fn apply_axis_modifier(input: i32, _output: &Axis, modifier: &mut AxisToAxisModifier) -> i32 {
    match modifier {
        //TODO: deadzone jumping --> scale value inside deadzone
        AxisToAxisModifier::Parameterized { params } => {
            let input_f32 = match &mut params.avg_filter {
                Some(filter_len) => {
                    let head = &mut params.avg_data.0;
                    let data = &mut params.avg_data.1;

                    if *head >= data.len() {
                        data.push(input)
                    } else {
                        data[*head] = input;
                    }

                    *head += 1;
                    if *head >= *filter_len {
                        *head = 0;
                    }

                    let count = data.len() as f32;
                    let sum = data.iter().sum::<i32>() as f32;
                    sum / count
                }
                None => input as f32,
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

            let linearity_value = offset_value.signum()
                * (offset_value / 32767.0).abs().powf(params.linearity)
                * 32767.0;

            linearity_value.floor() as i32
        }
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
