use crate::error::Error;
use egui::{ClippedPrimitive, TexturesDelta};
use egui_renderer::EguiRenderer;
use log::error;
use nalgebra_glm::Mat4;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::result::Result;
use vku::ash::vk::*;
use vku::*;
use winit::window::Window;

pub mod egui_color_test;
pub mod egui_renderer;
pub mod push_constants;
pub mod vertex;
pub use egui_color_test::ColorTest;

const MAX_FRAMES_IN_FLIGHT: usize = 1;

pub(crate) struct Graphics {
    vk_init: VkInit,
    setup_fence: Fence,
    setup_cmd_pool: CommandPool,
    setup_cmd_buffer: CommandBuffer,
    graphics_cmd_pool: CommandPool,
    graphics_cmd_buffers: Vec<CommandBuffer>,
    in_flight_fences: Vec<Fence>,
    image_acquired_semaphores: Vec<Semaphore>,
    render_complete_semaphores: Vec<Semaphore>,
    egui_renderer: EguiRenderer,
    frame: usize,
}

impl Graphics {
    #[profiling::function]
    pub(crate) fn new(window: &Window) -> Result<Self, Error> {
        let mut vk_init_create_info = if cfg!(debug_assertions) {
            VkInitCreateInfo::debug_vk_1_3()
        } else {
            VkInitCreateInfo::dist_vk_1_3()
        };

        vk_init_create_info.request_img_count = MAX_FRAMES_IN_FLIGHT as u32;
        vk_init_create_info.present_mode = PresentModeKHR::FIFO;

        let vk_init = VkInit::new(
            Some(&window.raw_display_handle()),
            Some(&window.raw_window_handle()),
            Some(window.inner_size().into()),
            vk_init_create_info,
        )?;

        let setup_fence = vk_init.create_fence()?;
        vk_init.set_debug_object_name(
            setup_fence.as_raw(),
            ObjectType::FENCE,
            String::from("RVM_Setup_Fence"),
        )?;

        let setup_cmd_pool = vk_init.create_cmd_pool(CmdType::Any)?;
        vk_init.set_debug_object_name(
            setup_cmd_pool.as_raw(),
            ObjectType::COMMAND_POOL,
            String::from("RVM_Setup_Cmd_Pool"),
        )?;

        let setup_cmd_buffer = vk_init.create_command_buffers(&setup_cmd_pool, 1)?[0];
        vk_init.set_debug_object_name(
            setup_cmd_buffer.as_raw(),
            ObjectType::COMMAND_BUFFER,
            String::from("RVM_Setup_Cmd_Buffer"),
        )?;

        let graphics_cmd_pool = vk_init.create_cmd_pool(CmdType::Any)?;
        vk_init.set_debug_object_name(
            graphics_cmd_pool.as_raw(),
            ObjectType::COMMAND_POOL,
            String::from("RVM_Graphics_Cmd_Pool"),
        )?;

        let graphics_cmd_buffers =
            vk_init.create_command_buffers(&graphics_cmd_pool, MAX_FRAMES_IN_FLIGHT as u32)?;
        let in_flight_fences = vk_init.create_fences(MAX_FRAMES_IN_FLIGHT)?;
        let image_acquired_semaphores = vk_init.create_semaphores(MAX_FRAMES_IN_FLIGHT)?;
        let render_complete_semaphores = vk_init.create_semaphores(MAX_FRAMES_IN_FLIGHT)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vk_init.set_debug_object_name(
                graphics_cmd_buffers[i].as_raw(),
                ObjectType::COMMAND_BUFFER,
                format!("RVM_Graphics_Cmd_Buffer_{i}"),
            )?;
            vk_init.set_debug_object_name(
                in_flight_fences[i].as_raw(),
                ObjectType::FENCE,
                format!("RVM_In_Flight_Fence_{i}"),
            )?;
            vk_init.set_debug_object_name(
                image_acquired_semaphores[i].as_raw(),
                ObjectType::SEMAPHORE,
                format!("RVM_Image_Acquired_Semaphore_{i}"),
            )?;
            vk_init.set_debug_object_name(
                render_complete_semaphores[i].as_raw(),
                ObjectType::SEMAPHORE,
                format!("RVM_Render_Complete_Semaphore_{i}"),
            )?;
        }

        let egui_renderer = EguiRenderer::new(&vk_init, MAX_FRAMES_IN_FLIGHT)?;

        vk_init.wait_on_fence_and_reset(Some(&setup_fence), &[&setup_cmd_buffer])?;
        vk_init.begin_cmd_buffer(&setup_cmd_buffer)?;

        vk_init.end_and_submit_cmd_buffer(
            &setup_cmd_buffer,
            CmdType::Any,
            &setup_fence,
            &[],
            &[],
            &[],
        )?;

        vk_init.wait_on_fence_and_reset(Some(&setup_fence), &[&setup_cmd_buffer])?;

        Ok(Self {
            vk_init,
            setup_fence,
            setup_cmd_pool,
            setup_cmd_buffer,
            graphics_cmd_pool,
            graphics_cmd_buffers,
            in_flight_fences,
            image_acquired_semaphores,
            render_complete_semaphores,
            egui_renderer,
            frame: 0,
        })
    }

    #[profiling::function]
    pub(crate) fn destroy(&mut self) -> Result<(), Error> {
        self.vk_init.wait_device_idle()?;
        self.egui_renderer.destroy(&self.vk_init)?;
        self.vk_init.destroy_fence(&self.setup_fence)?;
        self.vk_init.destroy_cmd_pool(&self.setup_cmd_pool)?;
        self.vk_init.destroy_cmd_pool(&self.graphics_cmd_pool)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            self.vk_init.destroy_fence(&self.in_flight_fences[i])?;
            self.vk_init
                .destroy_semaphore(&self.image_acquired_semaphores[i])?;
            self.vk_init
                .destroy_semaphore(&self.render_complete_semaphores[i])?;
        }

        self.vk_init.destroy()?;

        Ok(())
    }

    #[allow(clippy::modulo_one)] // flexible code to change frames in flight > 1
    #[profiling::function]
    pub(crate) fn update(
        &mut self,
        images_delta: TexturesDelta,
        clipped_primitives: Vec<ClippedPrimitive>,
        ui_to_ndc: Mat4,
    ) -> Result<(), Error> {
        let img_acquired_sem = self.image_acquired_semaphores[self.frame];
        let in_flight_fence = self.in_flight_fences[self.frame];
        let graphics_cmd_buffer = self.graphics_cmd_buffers[self.frame];
        let render_complete_sem = self.render_complete_semaphores[self.frame];

        let (swapchain_image_index, _swapchain_image, _swapchain_image_view, sub_optimal) = {
            profiling::scope!("Graphics::Update::AcquireImage");
            self.vk_init
                .acquire_next_swapchain_image(img_acquired_sem)?
        };

        if sub_optimal {
            error!("sub optimal swapchain");
        }

        {
            profiling::scope!("Graphics::Update::ResetInFlightFence");
            self.vk_init
                .wait_on_fence_and_reset(Some(&in_flight_fence), &[&graphics_cmd_buffer])?;
        }

        self.vk_init.begin_cmd_buffer(&graphics_cmd_buffer)?;

        self.egui_renderer.input(
            &self.vk_init,
            &graphics_cmd_buffer,
            clipped_primitives,
            images_delta,
            swapchain_image_index,
        )?;

        self.egui_renderer.draw(
            &self.vk_init,
            &graphics_cmd_buffer,
            ui_to_ndc,
            swapchain_image_index,
        )?;

        {
            profiling::scope!("Graphics::Update::EndAndSubmit");
            self.vk_init.end_and_submit_cmd_buffer(
                &graphics_cmd_buffer,
                CmdType::Any,
                &in_flight_fence,
                &[img_acquired_sem],
                &[render_complete_sem],
                &[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            )?;
        }

        {
            profiling::scope!("Graphics::Update::Present");
            self.vk_init
                .present(&render_complete_sem, swapchain_image_index)?;
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    #[profiling::function]
    pub(crate) fn on_resize(&mut self, window: &Window, new_size: [u32; 2]) -> Result<(), Error> {
        self.vk_init.on_resize(
            &window.raw_display_handle(),
            &window.raw_window_handle(),
            new_size,
        )?;

        self.egui_renderer.on_resize(&self.vk_init)?;

        self.transition_render_resources_before_first_usage()?;

        Ok(())
    }

    #[profiling::function]
    fn transition_render_resources_before_first_usage(&mut self) -> Result<(), Error> {
        self.vk_init.begin_cmd_buffer(&self.setup_cmd_buffer)?;

        let buffer_barriers2: Vec<BufferMemoryBarrier2> = vec![];
        let mut image_barriers2: Vec<ImageMemoryBarrier2> = vec![];

        //transition all swapchain images into PRESENT_SRC_KHR before first usage
        for swapchain_image in &self.vk_init.head().swapchain_images {
            let swapchain_layout_attachment_barrier = ImageMemoryBarrier2::builder()
                .image(*swapchain_image)
                .src_stage_mask(PipelineStageFlags2::TOP_OF_PIPE)
                .dst_stage_mask(PipelineStageFlags2::TOP_OF_PIPE)
                .src_access_mask(AccessFlags2::NONE)
                .dst_access_mask(AccessFlags2::NONE)
                .old_layout(ImageLayout::UNDEFINED)
                .new_layout(ImageLayout::PRESENT_SRC_KHR)
                .subresource_range(ImageSubresourceRange {
                    aspect_mask: ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                })
                .build();

            image_barriers2.push(swapchain_layout_attachment_barrier);
        }

        self.vk_init.cmd_pipeline_barrier2(
            &self.setup_cmd_buffer,
            &image_barriers2,
            &buffer_barriers2,
        );

        self.vk_init.end_and_submit_cmd_buffer(
            &self.setup_cmd_buffer,
            CmdType::Any,
            &self.setup_fence,
            &[],
            &[],
            &[],
        )?;

        self.vk_init
            .wait_on_fence_and_reset(Some(&self.setup_fence), &[&self.setup_cmd_buffer])?;

        Ok(())
    }
}
