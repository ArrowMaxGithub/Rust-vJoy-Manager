use std::mem::size_of;
use vku::ash::vk::*;
use vku::pipeline_builder::VertexConvert;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UIVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [u8; 4],
}

impl VertexConvert for UIVertex {
    fn binding_desc() -> Vec<VertexInputBindingDescription> {
        vec![VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<UIVertex>() as u32)
            .input_rate(VertexInputRate::VERTEX)
            .build()]
    }

    fn attrib_desc() -> Vec<VertexInputAttributeDescription> {
        vec![
            VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(Format::R32G32_SFLOAT)
                .offset(0)
                .build(),
            VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(Format::R32G32_SFLOAT)
                .offset(8)
                .build(),
            VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(Format::R8G8B8A8_UNORM)
                .offset(16)
                .build(),
        ]
    }
}
