use super::push_constants::PushConstants;
use super::vertex::UIVertex;
use egui::epaint::{Primitive, ImageDelta};
use egui::{ClippedPrimitive, TextureId, TexturesDelta, ImageData};
use log::info;
use nalgebra_glm::Mat4;
use std::collections::HashMap;
use vku::ash::vk::*;
use vku::*;
use std::result::Result;
use crate::error::Error;

pub(crate) struct EguiRenderer {
    base_renderer: BaseRenderer,
    clipped_primitives: Vec<ClippedPrimitive>,
    images: HashMap<TextureId, (VMAImage, DescriptorSet)>,
}

impl EguiRenderer {
    #[profiling::function]
    pub(crate) fn new(vk_init: &VkInit, frames_in_flight: usize) -> Result<Self, Error> {
        let create_info = RendererCreateInfo {
            initial_buffer_length: 1024 * 1024,
            frames_in_flight,
            topology: PrimitiveTopology::TRIANGLE_LIST,
            blend_mode: BlendMode::PremultipliedTransparency,
            vertex_code_path: String::from("./assets/shaders/compiled/egui.vert.spv"),
            fragment_code_path: String::from("./assets/shaders/compiled/egui.frag.spv"),
            additional_usage_index_buffer: BufferUsageFlags::empty(),
            additional_usage_vertex_buffer: BufferUsageFlags::empty(),
            debug_name: String::from("Hotas_EguiRenderer"),
        };
        let base_renderer =
            vk_init.create_base_renderer::<u32, UIVertex, PushConstants>(&create_info)?;

        let clipped_primitives = Vec::new();
        let images = HashMap::new();

        Ok(Self {
            base_renderer,
            clipped_primitives,
            images,
        })
    }

    #[profiling::function]
    pub(crate) fn destroy(&self, vk_init: &VkInit) -> Result<(), Error> {
        for (img, _) in self.images.values() {
            img.destroy(vk_init)?;
        }
        vk_init.destroy_base_renderer(&self.base_renderer)?;
        Ok(())
    }

    #[profiling::function]
    pub(crate) fn input(
        &mut self,
        vk_init: &VkInit,
        cmd_buffer: &CommandBuffer,
        clipped_primitives: Vec<ClippedPrimitive>,
        images_delta: TexturesDelta,
        frame: usize
    ) -> Result<(), Error> {
        self.clipped_primitives = clipped_primitives;

        for delta in &images_delta.set {
            self.update_image(vk_init, &delta.0, &delta.1, cmd_buffer)?;
        }

        let indices: Vec<u32> = self
            .clipped_primitives
            .iter()
            .filter_map(|clip| match &clip.primitive {
                Primitive::Mesh(mesh) => Some(mesh.indices.to_owned()),
                Primitive::Callback(_) => None,
            })
            .flatten()
            .collect();

        let vertices: Vec<egui_winit::egui::epaint::Vertex> = self
            .clipped_primitives
            .iter()
            .filter_map(|clip| match &clip.primitive {
                Primitive::Mesh(mesh) => Some(mesh.vertices.to_owned()),
                Primitive::Callback(_) => None,
            })
            .flatten()
            .collect();

        let index_buffer = &self.base_renderer.index_buffers[frame];
        let vertex_buffer = &self.base_renderer.vertex_buffers[frame];

        {
            profiling::scope!("EguiRenderer::Input::SetData");
            index_buffer.set_data(&indices)?;
            vertex_buffer.set_data(&vertices)?;
        }

        Ok(())
    }

    #[profiling::function]
    pub(crate) fn draw(
        &mut self,
        vk_init: &VkInit,
        cmd_buffer: &CommandBuffer,
        ui_space_matrix: Mat4,
        frame: usize,
    ) -> Result<(), Error> {
        let push = PushConstants {
            mat0: ui_space_matrix,
            vec0: [0.0; 4].into(),
            vec1: [0.0; 4].into(),
            vec2: [0.0; 4].into(),
            vec3: [0.0; 4].into(),
        }
        .as_bytes();

        let index_buffer = &self.base_renderer.index_buffers[frame];
        let vertex_buffer = &self.base_renderer.vertex_buffers[frame];
        unsafe {
            vk_init.device.cmd_bind_pipeline(
                *cmd_buffer,
                PipelineBindPoint::GRAPHICS,
                self.base_renderer.pipeline,
            );
            vk_init.device.cmd_bind_index_buffer(
                *cmd_buffer,
                index_buffer.buffer,
                0,
                IndexType::UINT32,
            );
            vk_init
                .device
                .cmd_bind_vertex_buffers(*cmd_buffer, 0, &[vertex_buffer.buffer], &[0]);
            vk_init.device.cmd_push_constants(
                *cmd_buffer,
                self.base_renderer.pipeline_layout,
                ShaderStageFlags::VERTEX,
                0,
                &push,
            );
        }

        let mut index_offset = 0;
        let mut vertex_offset = 0;

        for (_, cp) in self.clipped_primitives.iter().enumerate() {
            let Primitive::Mesh(mesh) = &cp.primitive else { return Err(Error::GraphicsError("render callbacks not supported.".to_string()))};
            let clip_rect = cp.clip_rect;
            let texture_id = mesh.texture_id;
            let (_, desc_set) = self
                .images
                .get(&texture_id)
                .expect("Texture no longer exists");
            let index_count = mesh.indices.len() as u32;
            let vertex_count = mesh.vertices.len() as u32;

            //Set clip rect
            let scissor = Rect2D {
                offset: Offset2D {
                    x: clip_rect.min.x.max(0.0) as _,
                    y: clip_rect.min.y.max(0.0) as _,
                },
                extent: Extent2D {
                    width: clip_rect.width() as _,
                    height: clip_rect.height() as _,
                },
            };
            let current_extent = vk_init.head().surface_info.current_extent;
            let viewport = Viewport::builder()
                .width(current_extent.width as f32)
                .height(current_extent.height as f32)
                .max_depth(1.0)
                .build();

            unsafe {
                vk_init.device.cmd_set_scissor(*cmd_buffer, 0, &[scissor]);
                vk_init.device.cmd_set_viewport(*cmd_buffer, 0, &[viewport]);
                vk_init.device.cmd_bind_descriptor_sets(
                    *cmd_buffer,
                    PipelineBindPoint::GRAPHICS,
                    self.base_renderer.pipeline_layout,
                    0,
                    &[*desc_set],
                    &[],
                );

                vk_init.device.cmd_draw_indexed(
                    *cmd_buffer,
                    index_count,
                    1,
                    index_offset,
                    vertex_offset,
                    0,
                );

                index_offset += index_count;
                vertex_offset += vertex_count as i32;
            }
        }

        Ok(())
    }

    #[profiling::function]
    fn update_image(
        &mut self,
        vk_init: &VkInit,
        texture_id: &TextureId,
        image_delta: &ImageDelta,
        cmd_buffer: &CommandBuffer,
    ) -> Result<(), Error> {
        let extent = Extent3D {
            width: image_delta.image.width() as u32,
            height: image_delta.image.height() as u32,
            depth: 1,
        };

        //Create new image from image delta and transition to transfer_dst
        let new_img =
            self.create_new_image(vk_init, extent, texture_id, image_delta, cmd_buffer)?;

        //Blit from new img to existing img or insert new image without any operation
        self.blit_or_insert(
            vk_init,
            cmd_buffer,
            new_img,
            texture_id,
            image_delta,
            extent,
        )?;

        //Get updated or inserted image
        let image = self.images.get_mut(texture_id).unwrap();

        //Transition updated image to shader_read and update the desc set
        let shader_read_barrier = image.0.get_image_layout_transition_barrier2(
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            None,
            None,
        )?;
        vk_init.cmd_pipeline_barrier2(cmd_buffer, &[shader_read_barrier], &[]);

        let image_desc_info = DescriptorImageInfo {
            image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: image.0.image_view,
            sampler: self.base_renderer.sampler,
        };

        let write_desc_set = [WriteDescriptorSet {
            dst_set: image.1,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &image_desc_info,
            ..Default::default()
        }];

        unsafe { vk_init.device.update_descriptor_sets(&write_desc_set, &[]) };

        Ok(())
    }

    #[profiling::function]
    fn blit_or_insert(
        &mut self,
        vk_init: &VkInit,
        cmd_buffer: &CommandBuffer,
        mut new_img: (VMAImage, DescriptorSet),
        texture_id: &TextureId,
        image_delta: &ImageDelta,
        extent: Extent3D,
    ) -> Result<(), Error> {
        if let Some(existing_img) = self.images.get_mut(texture_id) {
            let transfer_barrier = new_img.0.get_image_layout_transition_barrier2(
                ImageLayout::TRANSFER_SRC_OPTIMAL,
                None,
                None,
            )?;
            vk_init.cmd_pipeline_barrier2(cmd_buffer, &[transfer_barrier], &[]);

            let transfer_barrier = existing_img.0.get_image_layout_transition_barrier2(
                ImageLayout::TRANSFER_DST_OPTIMAL,
                None,
                None,
            )?;
            vk_init.cmd_pipeline_barrier2(cmd_buffer, &[transfer_barrier], &[]);

            let src_offset = image_delta.pos.unwrap_or([0, 0]);

            let regions = ImageBlit::builder()
                .src_subresource(ImageSubresourceLayers {
                    aspect_mask: ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_offsets([
                    Offset3D { x: 0, y: 0, z: 0 },
                    Offset3D {
                        x: extent.width as i32,
                        y: extent.height as i32,
                        z: 1,
                    },
                ])
                .dst_subresource(ImageSubresourceLayers {
                    aspect_mask: ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .dst_offsets([
                    Offset3D {
                        x: src_offset[0] as i32,
                        y: src_offset[1] as i32,
                        z: 0,
                    },
                    Offset3D {
                        x: src_offset[0] as i32 + extent.width as i32,
                        y: src_offset[1] as i32 + extent.height as i32,
                        z: 1,
                    },
                ])
                .build();

            unsafe {
                vk_init.device.cmd_blit_image(
                    *cmd_buffer,
                    new_img.0.image,
                    ImageLayout::TRANSFER_SRC_OPTIMAL,
                    existing_img.0.image,
                    ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[regions],
                    Filter::NEAREST,
                );
            }
        } else {
            assert!(
                image_delta.pos.is_none(),
                "Partial img update with no existing img"
            );
            self.images.insert(*texture_id, new_img);
        }

        Ok(())
    }

    #[profiling::function]
    fn create_new_image(
        &mut self,
        vk_init: &VkInit,
        extent: Extent3D,
        texture_id: &TextureId,
        image_delta: &ImageDelta,
        cmd_buffer: &CommandBuffer,
    ) -> Result<(VMAImage, DescriptorSet), Error> {
        let mut img =
            vk_init.create_empty_image(extent, Format::R8G8B8A8_SRGB, ImageAspectFlags::COLOR)?;
        img.set_debug_object_name(vk_init, format!("VKU_EguiRenderer_{:?}", texture_id))?;
        info!("Creating image VKU_EguiRenderer_{:?}", texture_id);

        let set_layouts = [self.base_renderer.sampled_image_desc_set_layout];
        let image_desc_set_alloc_info = DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.base_renderer.descriptor_pool)
            .set_layouts(&set_layouts)
            .build();

        let img_desc_set = unsafe {
            vk_init
                .device
                .allocate_descriptor_sets(&image_desc_set_alloc_info)?[0]
        };

        vk_init.set_debug_object_name(
            img_desc_set.as_raw(),
            ObjectType::DESCRIPTOR_SET,
            format!("VKU_EguiRenderer_{:?}_Desc_Set", texture_id),
        )?;

        let data = match &image_delta.image {
            ImageData::Color(data) => data
                .pixels
                .iter()
                .flat_map(|c| c.to_array())
                .collect::<Vec<_>>(),
            ImageData::Font(data) => data
                .srgba_pixels(None)
                .flat_map(|c| c.to_array())
                .collect::<Vec<_>>(),
        };

        let transfer_barrier = img.get_image_layout_transition_barrier2(
            ImageLayout::TRANSFER_DST_OPTIMAL,
            None,
            None,
        )?;

        //Load data into new img
        vk_init.cmd_pipeline_barrier2(cmd_buffer, &[transfer_barrier], &[]);
        img.set_staging_data(&data)?;
        img.enque_copy_from_staging_buffer_to_image(vk_init, cmd_buffer);

        Ok((img, img_desc_set))
    }
}
