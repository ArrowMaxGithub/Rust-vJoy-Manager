use super::push_constants::PushConstants;
use super::vertex::UIVertex;
use crate::error::Error;
use egui::epaint::{ImageDelta, Primitive, Vertex};
use egui::{ClippedPrimitive, ImageData, Rect, TextureId, TexturesDelta};
use log::trace;
use nalgebra_glm::Mat4;
use std::collections::HashMap;
use std::mem::size_of;
use std::result::Result;
use vku::ash::vk::*;
use vku::pipeline_builder::{BlendMode, DepthInfo, StencilInfo, VKUPipeline};
use vku::*;

#[derive(Debug, Clone, Copy)]
struct MeshDrawInfo {
    pub tex_id: TextureId,
    pub indices_count: u32,
    pub vertices_count: i32,
    pub rect: Rect,
}

impl Default for MeshDrawInfo {
    fn default() -> Self {
        Self {
            tex_id: TextureId::default(),
            indices_count: 0,
            vertices_count: 0,
            rect: Rect {
                min: egui::Pos2 { x: 0.0, y: 0.0 },
                max: egui::Pos2 { x: 0.0, y: 0.0 },
            },
        }
    }
}

pub(crate) struct EguiRenderer {
    index_buffers: Vec<VMABuffer>,
    vertex_buffers: Vec<VMABuffer>,
    framebuffers: Vec<Framebuffer>,

    pipeline: VKUPipeline,

    desc_pool: DescriptorPool,
    sampler: Sampler,

    images: HashMap<TextureId, (VMAImage, DescriptorSet)>,
    mesh_draw_infos: Vec<MeshDrawInfo>,
    mesh_draw_count: usize,
}

impl EguiRenderer {
    #[profiling::function]
    pub(crate) fn new(vk_init: &VkInit, frames_in_flight: usize) -> Result<Self, Error> {
        let index_buffers = vk_init.create_cpu_to_gpu_buffers(
            1024 * 1024 * size_of::<u32>(),
            BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
            frames_in_flight,
        )?;
        for (i, index_buffer) in index_buffers.iter().enumerate() {
            index_buffer
                .set_debug_object_name(vk_init, format!("RVM_Egui_Renderer_Index_Buffer_{i}"))?;
        }

        let vertex_buffers = vk_init.create_cpu_to_gpu_buffers(
            1024 * 1024 * size_of::<UIVertex>(),
            BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
            frames_in_flight,
        )?;
        for (i, vertex_buffer) in vertex_buffers.iter().enumerate() {
            vertex_buffer
                .set_debug_object_name(vk_init, format!("RVM_Egui_Renderer_Vertex_Buffer_{i}"))?;
        }

        let desc_pool_size = [DescriptorPoolSize {
            ty: DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 64,
        }];

        let desc_pool_info = DescriptorPoolCreateInfo::builder()
            .pool_sizes(&desc_pool_size)
            .max_sets(64)
            .flags(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);

        let desc_pool = unsafe {
            vk_init
                .device
                .create_descriptor_pool(&desc_pool_info, None)?
        };
        vk_init.set_debug_object_name(
            desc_pool.as_raw(),
            ObjectType::DESCRIPTOR_POOL,
            "RVM_Egui_Renderer_Desc_Pool".to_string(),
        )?;

        let sampler_info = SamplerCreateInfo::builder()
            .mag_filter(Filter::LINEAR)
            .min_filter(Filter::LINEAR)
            .address_mode_u(SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(SamplerAddressMode::CLAMP_TO_EDGE)
            .mipmap_mode(SamplerMipmapMode::LINEAR);

        let sampler = unsafe { vk_init.device.create_sampler(&sampler_info, None)? };
        vk_init.set_debug_object_name(
            sampler.as_raw(),
            ObjectType::SAMPLER,
            "RVM_Egui_Renderer_Sampler".to_string(),
        )?;

        let pipeline = VKUPipeline::builder()
            .with_vertex::<UIVertex>(PrimitiveTopology::TRIANGLE_LIST)
            .with_tesselation(1)
            .with_viewports_scissors(&[Viewport::default()], &[Rect2D::default()]) // using dynamic viewport/scissor later
            .with_rasterization(PolygonMode::FILL, CullModeFlags::NONE)
            .with_multisample(SampleCountFlags::TYPE_1)
            .with_depthstencil(DepthInfo::default(), StencilInfo::default())
            .with_colorblends(&[BlendMode::PremultipliedTransparency])
            .with_dynamic(&[DynamicState::VIEWPORT, DynamicState::SCISSOR])
            .with_push_constants::<PushConstants>()
            .with_descriptors(&[(
                DescriptorType::COMBINED_IMAGE_SAMPLER,
                ShaderStageFlags::FRAGMENT,
                1,
            )])
            .push_shader_stage(
                &vk_init.device,
                ShaderStageFlags::VERTEX,
                "assets/shaders/egui.vert.spv",
                &[],
            )
            .push_shader_stage(
                &vk_init.device,
                ShaderStageFlags::FRAGMENT,
                "assets/shaders/egui.frag.spv",
                &[],
            )
            .with_render_pass(
                &[AttachmentDescription::builder()
                    .format(vk_init.head().surface_info.color_format.format)
                    .samples(SampleCountFlags::TYPE_1)
                    .load_op(AttachmentLoadOp::CLEAR)
                    .store_op(AttachmentStoreOp::STORE)
                    .initial_layout(ImageLayout::PRESENT_SRC_KHR)
                    .final_layout(ImageLayout::PRESENT_SRC_KHR)
                    .build()],
                &[SubpassDescription::builder()
                    .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
                    .color_attachments(&[AttachmentReference {
                        attachment: 0,
                        layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    }])
                    .build()],
                &[
                    // swapchain acq barrier
                    SubpassDependency::builder()
                        .src_subpass(SUBPASS_EXTERNAL)
                        .dst_subpass(0)
                        .src_stage_mask(PipelineStageFlags::TOP_OF_PIPE)
                        .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                        .src_access_mask(AccessFlags::NONE)
                        .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
                        .build(),
                    // swapchain present barrier
                    SubpassDependency::builder()
                        .src_subpass(0)
                        .dst_subpass(SUBPASS_EXTERNAL)
                        .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                        .dst_stage_mask(PipelineStageFlags::NONE)
                        .src_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
                        .dst_access_mask(AccessFlags::NONE)
                        .build(),
                ],
            )
            .build(vk_init, "RVM_Egui_Renderer")
            .unwrap();

        let images = HashMap::new();
        let mesh_draw_infos = vec![MeshDrawInfo::default(); 1024];

        Ok(Self {
            index_buffers,
            vertex_buffers,
            framebuffers: Vec::new(),
            pipeline,
            desc_pool,
            sampler,
            images,
            mesh_draw_infos,
            mesh_draw_count: 0,
        })
    }

    #[profiling::function]
    pub(crate) fn destroy(&mut self, vk_init: &VkInit) -> Result<(), Error> {
        for (img, _) in self.images.values_mut() {
            img.destroy(&vk_init.device, &vk_init.allocator)?;
        }

        for framebuffer in &self.framebuffers {
            unsafe { vk_init.device.destroy_framebuffer(*framebuffer, None) }
        }

        for buffer in &mut self.index_buffers {
            buffer.destroy(&vk_init.allocator)?;
        }
        for buffer in &mut self.vertex_buffers {
            buffer.destroy(&vk_init.allocator)?;
        }

        unsafe {
            vk_init.device.destroy_descriptor_pool(self.desc_pool, None);
            vk_init.device.destroy_sampler(self.sampler, None);
        }

        self.pipeline.destroy(&vk_init.device)?;
        Ok(())
    }

    #[profiling::function]
    pub(crate) fn input(
        &mut self,
        vk_init: &VkInit,
        cmd_buffer: &CommandBuffer,
        clipped_primitives: Vec<ClippedPrimitive>,
        images_delta: TexturesDelta,
        frame: usize,
    ) -> Result<(), Error> {
        for delta in &images_delta.set {
            self.update_image(vk_init, &delta.0, &delta.1, cmd_buffer)?;
        }

        let index_buffer = &self.index_buffers[frame];
        let vertex_buffer = &self.vertex_buffers[frame];

        {
            let mut indices_count = 0;
            let mut vertices_count = 0;
            let mut draw_infos_count = 0;

            profiling::scope!("EguiRenderer::Input::DataIterator");
            for clip in clipped_primitives.into_iter() {
                let Primitive::Mesh(mesh) = clip.primitive else {
                    panic!("render callbacks are not supported");
                };

                let mesh_draw_info = MeshDrawInfo {
                    tex_id: mesh.texture_id,
                    indices_count: mesh.indices.len() as u32,
                    vertices_count: mesh.vertices.len() as i32,
                    rect: clip.clip_rect,
                };

                unsafe {
                    let ptr = index_buffer.allocation_info.mapped_data as *mut u32;
                    let ptr = ptr.add(indices_count);
                    ptr.copy_from_nonoverlapping(mesh.indices.as_ptr(), mesh.indices.len());
                }
                unsafe {
                    let ptr = vertex_buffer.allocation_info.mapped_data as *mut Vertex;
                    let ptr = ptr.add(vertices_count);
                    ptr.copy_from_nonoverlapping(mesh.vertices.as_ptr(), mesh.vertices.len());
                }

                self.mesh_draw_infos[draw_infos_count] = mesh_draw_info;

                indices_count += mesh.indices.len();
                vertices_count += mesh.vertices.len();
                draw_infos_count += 1;
            }

            self.mesh_draw_count = draw_infos_count;
        }

        Ok(())
    }

    #[profiling::function]
    pub(crate) fn draw(
        &mut self,
        vk_init: &VkInit,
        cmd_buffer: &CommandBuffer,
        ui_to_ndc_mat: Mat4,
        frame: usize,
        swapchain_index: usize,
    ) -> Result<(), Error> {
        let clear_color_value = ClearValue {
            color: ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        };

        let render_pass_begin_info = RenderPassBeginInfo::builder()
            .render_pass(self.pipeline.renderpass)
            .framebuffer(self.framebuffers[swapchain_index])
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: vk_init.head().surface_info.current_extent,
            })
            .clear_values(&[clear_color_value])
            .build();

        unsafe {
            vk_init.device.cmd_begin_render_pass(
                *cmd_buffer,
                &render_pass_begin_info,
                SubpassContents::INLINE,
            );
        }

        let push = PushConstants {
            mat0: ui_to_ndc_mat,
            vec0: [0.0; 4].into(),
            vec1: [0.0; 4].into(),
            vec2: [0.0; 4].into(),
            vec3: [0.0; 4].into(),
        }
        .as_bytes();

        let index_buffer = &self.index_buffers[frame];
        let vertex_buffer = &self.vertex_buffers[frame];
        unsafe {
            vk_init.device.cmd_bind_pipeline(
                *cmd_buffer,
                PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
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
                self.pipeline.layout,
                ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT,
                0,
                &push,
            );
        }

        let mut index_offset = 0;
        let mut vertex_offset = 0;

        for info in &self.mesh_draw_infos[0..self.mesh_draw_count] {
            let (_, desc_set) = self
                .images
                .get(&info.tex_id)
                .expect("Texture no longer exists");

            //Set clip rect
            let scissor = Rect2D {
                offset: Offset2D {
                    x: info.rect.min.x.max(0.0) as _,
                    y: info.rect.min.y.max(0.0) as _,
                },
                extent: Extent2D {
                    width: info.rect.width() as _,
                    height: info.rect.height() as _,
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
                    self.pipeline.layout,
                    0,
                    &[*desc_set],
                    &[],
                );

                vk_init.device.cmd_draw_indexed(
                    *cmd_buffer,
                    info.indices_count,
                    1,
                    index_offset,
                    vertex_offset,
                    0,
                );

                index_offset += info.indices_count;
                vertex_offset += info.vertices_count;
            }
        }

        unsafe {
            vk_init.device.cmd_end_render_pass(*cmd_buffer);
        }

        Ok(())
    }

    #[profiling::function]
    pub(crate) fn on_resize(&mut self, vk_init: &VkInit) -> Result<(), Error> {
        for framebuffer in &self.framebuffers {
            unsafe {
                vk_init.device.destroy_framebuffer(*framebuffer, None);
            }
        }

        self.framebuffers = vk_init
            .head()
            .swapchain_image_views
            .iter()
            .flat_map(|image_view| {
                let create_info = FramebufferCreateInfo::builder()
                    .render_pass(self.pipeline.renderpass)
                    .attachments(&[*image_view])
                    .width(vk_init.head().surface_info.current_extent.width)
                    .height(vk_init.head().surface_info.current_extent.height)
                    .layers(1)
                    .build();
                unsafe { vk_init.device.create_framebuffer(&create_info, None) }
            })
            .collect();

        for (i, framebuffer) in self.framebuffers.iter().enumerate() {
            vk_init.set_debug_object_name(
                framebuffer.as_raw(),
                ObjectType::FRAMEBUFFER,
                format!("RVM_Egui_Renderer_Framebuffer_{i}"),
            )?;
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
            sampler: self.sampler,
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
        trace!("Creating image VKU_EguiRenderer_{:?}", texture_id);

        let set_layouts = [self.pipeline.set_layout];
        let image_desc_set_alloc_info = DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.desc_pool)
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
        img.enque_copy_from_staging_buffer_to_image(&vk_init.device, cmd_buffer);

        Ok((img, img_desc_set))
    }
}
