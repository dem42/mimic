use crate::{
    buffers::{buffer::Buffer, index_buffer::IndexBuffer, memory, vertex_buffer::VertexBuffer},
    depth::depth_resource::DepthResource,
    devices::{
        logical_device::create_logical_device,
        physical_device::{get_physical_device_properties, pick_physical_device},
        queues::{QueueFamilyIndices, QueueMap, QueueType},
        requirements::DeviceRequirements,
    },
    drawing::{command_buffers, framebuffers, synchronization::SynchronizationContainer},
    graphics_pipeline::GraphicsPipeline,
    models::textured_model::{Mesh, MeshLoadingFlags},
    msaa::{multisampling::ColorResource, util::get_max_sample_count},
    presentation::{
        image_views::ImageViews,
        swap_chain::{SwapChainContainer, SwapChainSupportDetails},
    },
    textures::images::TextureImage,
    uniforms::{self, descriptors::DescriptorData},
    util::{
        self,
        debug::VulkanDebug,
        platform::SurfaceContainer,
        result::{Result, VulkanError},
        validation::VulkanValidation,
    },
    window::{WindowSize, WindowSurface},
};
use ash::{
    prelude::VkResult,
    vk,
};
use log::info;
use mimic_common::{apptime::AppTime, config::MimicConfig, texture::{FilesystemTextureSource, TextureSource}, uniforms::{StaticFnUniformSpec, UniformBufferObject, UniformSpec, UniformUpdateInput, update_uniform_buffer}};
use std::{
    convert::TryFrom,
    ffi::CString,
    path::{Path, PathBuf},
    ptr,
};
//////////////////////// Consts ///////////////////////
const REQUIRED_QUEUES: [QueueType; 2] = [
    QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS),
    QueueType::PresentQueue,
];
const DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
//////////////////////// Enums ///////////////////////
/// This enum informs us during which part of the draw-frame process a window resize happened
enum ResizeDetectedLocation {
    InAcquire,
    InPresent,
}
//////////////////////// Structs ///////////////////////
/// A structure that represents a vulkan application.
/// It exposes fields and methods that make it possible to communicate with the vulkan graphics API
pub struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    _validation: VulkanValidation,
    debug: VulkanDebug,
    surface_container: SurfaceContainer,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    logical_device: ash::Device,
    queue_indices: QueueFamilyIndices,
    queues: QueueMap,
    dependent_fields: SwapChainDependentFields,
    command_pool: vk::CommandPool,
    current_render_command: Option<RenderCommand>,
    sync_container: SynchronizationContainer,
    msaa_samples: vk::SampleCountFlags,
    /// resolve resource files
    pub resource_resolver: MimicConfig,
    /// This field is used to determine whether the window was resized.
    /// This is for example the case when the graphics display window was resized.
    pub window_resized: bool,
    /// This field is used to determine whether the display window was minimized.
    /// When the window is minimized then nothing needs to be rendered.
    pub window_minimized: bool,
}

struct RenderCommandSwapChainFields {
    uniform_buffers: Vec<Buffer>,
    descriptor_data: DescriptorData,
    command_buffers: Vec<vk::CommandBuffer>,
    graphics_pipeline: GraphicsPipeline,
    framebuffers: Vec<vk::Framebuffer>,
}

struct RenderCommand {
    vertex_shader_file: PathBuf,
    fragment_shader_file: PathBuf,
    uniform_spec: Box<dyn UniformSpec>,
    _model: Mesh,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture_image: TextureImage,
    uniform_descriptors: vk::DescriptorSetLayout,
    dependent_fields: RenderCommandSwapChainFields,
}

/// This structure wraps all the objects that depend on the swap-chain in order to be able to recreate them when the swap-chain images change.
/// Swap-chain is a series of framebuffers that can be drawn to and later presented to the graphics display.
/// The purpose is to allow double buffering so that the framebuffer isn't being written to while it is presented.
struct SwapChainDependentFields {
    swap_chain_container: SwapChainContainer,
    image_views_container: ImageViews,
    color_resource: ColorResource,
    depth_resource: DepthResource,
}
//////////////////////// Impls ///////////////////////
impl VulkanApp {
    /// Constructs a new `VulkanApp` with the provided window title and engine name.
    // It creates a swap-chain using the `window_surface` and `window_size`.
    pub fn new(
        window_title: &str,
        engine_name: &str,
        window_surface: &WindowSurface,
        window_size: &WindowSize,
        resource_resolver: MimicConfig,
    ) -> Result<Self> {
        let entry = unsafe { ash::Entry::load().unwrap() };
        let validation = VulkanValidation::enabled(util::validation::ValidationOptions::Verbose);
        // creating the instance is equivalent to initializing the vulkan library
        let instance = Self::create_instance(window_title, engine_name, &entry, &validation)?;
        let debug = VulkanDebug::new(&entry, &instance, &validation);
        // creating a surface to present images to
        let surface_container = util::platform::create_surface(&entry, &instance, window_surface)?;
        // pick the first graphics card that supports all the features we specified in instance
        let requirements = DeviceRequirements::new(
            &REQUIRED_QUEUES,
            &DEVICE_EXTENSIONS,
            is_swap_chain_adequate,
            is_device_supporting_features,
        );
        let physical_device = pick_physical_device(&instance, &surface_container, &requirements)?;
        let physical_device_properties =
            get_physical_device_properties(&instance, physical_device)?;
        let msaa_samples = get_max_sample_count(physical_device_properties);
        // create logical device and queues
        let queue_indices = QueueFamilyIndices::find(
            &instance,
            physical_device,
            &surface_container,
            &requirements,
        )?;
        let logical_device = create_logical_device(
            &instance,
            physical_device,
            &queue_indices,
            &requirements,
            &validation,
        )?;

        let sync_container = SynchronizationContainer::create(&logical_device)?;

        let command_pool = command_buffers::create_command_pool(&logical_device, &queue_indices)?;

        let queues = QueueMap::new(&queue_indices, &logical_device)?;

        let current_render_command: Option<RenderCommand> = None;

        let dependent_fields = Self::create_swapchain_dependent_fields(
            &instance,
            physical_device,
            &logical_device,
            &queue_indices,
            &surface_container,
            &command_pool,
            &queues,
            window_size,
            msaa_samples,
        )?;

        let result = Self {
            _entry: entry,
            instance,
            _validation: validation,
            debug,
            surface_container,
            physical_device,
            physical_device_properties,
            logical_device,
            queue_indices,
            queues,
            dependent_fields,
            command_pool,
            sync_container,
            msaa_samples,
            current_render_command,
            resource_resolver,
            window_resized: false,
            window_minimized: false,
        };

        Ok(result)
    }

    fn create_render_command_swap_chain_fields(
        vertex_shader_file: &Path,
        fragment_shader_file: &Path,
        uniform_spec: &Box<dyn UniformSpec>,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        texture_image: &TextureImage,
        vertex_buffer: &VertexBuffer,
        index_buffer: &IndexBuffer,
        command_pool: &vk::CommandPool,
        uniform_descriptors: vk::DescriptorSetLayout,
        swap_chain_dependent_fields: &SwapChainDependentFields,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<RenderCommandSwapChainFields> {
        let graphics_pipeline = GraphicsPipeline::new(
            vertex_shader_file,
            fragment_shader_file,
            instance,
            logical_device,
            physical_device,
            &swap_chain_dependent_fields.swap_chain_container,
            &uniform_descriptors,
            msaa_samples,
        )?;

        let framebuffers = framebuffers::create_framebuffers(
            logical_device,
            &graphics_pipeline,
            &swap_chain_dependent_fields.image_views_container,
            swap_chain_dependent_fields.depth_resource.depth_image_view,
            &swap_chain_dependent_fields.color_resource,
            &swap_chain_dependent_fields.swap_chain_container,
        )?;

        let uniform_buffers = uniforms::buffers::create_uniform_buffers(
            uniform_spec.uniform_buffer_size(),
            instance,
            physical_device,
            logical_device,
            &swap_chain_dependent_fields.swap_chain_container,
        )?;

        let descriptor_data = DescriptorData::new(
            &uniform_spec,
            logical_device,
            &swap_chain_dependent_fields.swap_chain_container,
            uniform_descriptors,
            &uniform_buffers,
            texture_image,
        )?;

        // command buffers are released when we destroy the pool
        let command_buffers = command_buffers::create_command_buffers(
            logical_device,
            command_pool,
            &framebuffers,
            &graphics_pipeline,
            &swap_chain_dependent_fields.swap_chain_container,
            vertex_buffer,
            index_buffer,
            &descriptor_data,
        )?;

        Ok(RenderCommandSwapChainFields {
            uniform_buffers,
            descriptor_data,
            command_buffers,
            graphics_pipeline,
            framebuffers,
        })
    }

    pub fn create_default_render_command(&mut self) -> Result<()> {
        let texture_path = self.resource_resolver
        .resolve_resource("res/backend/textures/viking_room.png")?;
        let texture_source = Box::new(FilesystemTextureSource::new(texture_path)?);

        self.create_render_command(
            texture_source,
            self.resource_resolver
                .resolve_resource("res/backend/models/viking_room.obj")?,
            self.resource_resolver
                .resolve_resource("res/backend/shaders/spv/simple_triangle.vert.spv")?,
            self.resource_resolver
                .resolve_resource("res/backend/shaders/spv/simple_triangle.frag.spv")?,
            Box::new(StaticFnUniformSpec::new::<UniformBufferObject>(update_uniform_buffer)),
        )
    }

    pub fn create_render_command(
        &mut self,
        texture_source: Box<dyn TextureSource>,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
        uniform_spec: Box<dyn UniformSpec>,
    ) -> Result<()> {
        // we are removing the previous render command so we must block until we are done using it and then clean it up
        unsafe {
            self.logical_device.device_wait_idle()?;
        }
        if let Some(render_command) = &mut self.current_render_command {
            unsafe {
                render_command.cleanup(&self.logical_device, self.command_pool);
            }
        }

        let texture_image = TextureImage::new(
            texture_source,
            &self.instance,
            self.physical_device,
            &self.logical_device,
            self.command_pool,
            &self.queues,
            &self.physical_device_properties,
        )?;

        let model = Mesh::new(model_file.as_path(), MeshLoadingFlags::INVERTED_UP)?;

        let vertex_buffer = VertexBuffer::new(
            &model.vertices,
            &self.instance,
            self.physical_device,
            &self.logical_device,
            self.command_pool,
            &self.queues,
        )?;

        let index_buffer = IndexBuffer::new(
            &model.indices,
            &self.instance,
            self.physical_device,
            &self.logical_device,
            self.command_pool,
            &self.queues,
        )?;

        let uniform_descriptors =
        uniforms::descriptors::create_descriptor_set_layout(&self.logical_device)?;

        let dependent_fields = Self::create_render_command_swap_chain_fields(
            vertex_shader_file.as_path(),
            fragment_shader_file.as_path(),
            &uniform_spec,
            &self.instance,
            self.physical_device,
            &self.logical_device,
            &texture_image,
            &vertex_buffer,
            &index_buffer,
            &self.command_pool,
            uniform_descriptors,
            &self.dependent_fields,
            self.msaa_samples,
        )?;

        self.current_render_command = Some(RenderCommand {
            vertex_shader_file,
            fragment_shader_file,
            uniform_spec,
            _model: model,
            vertex_buffer,
            index_buffer,
            texture_image,
            uniform_descriptors,
            dependent_fields,
        });

        Ok(())
    }

    /// Swapchain dependent fields are the ones that we need to recreate for example anytime that the window size changes
    fn create_swapchain_dependent_fields(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        queue_indices: &QueueFamilyIndices,
        surface_container: &SurfaceContainer,
        command_pool: &vk::CommandPool,
        queues: &QueueMap,
        window_size: &WindowSize,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<SwapChainDependentFields> {
        let swap_chain_container = SwapChainContainer::new(
            instance,
            physical_device,
            logical_device,
            surface_container,
            window_size,
            queue_indices,
        )?;

        let image_views_container = ImageViews::new(logical_device, &swap_chain_container)?;

        let color_resource = ColorResource::new(
            msaa_samples,
            instance,
            logical_device,
            physical_device,
            &swap_chain_container,
        )?;

        let depth_resource = DepthResource::new(
            msaa_samples,
            instance,
            logical_device,
            physical_device,
            &swap_chain_container,
            *command_pool,
            queues,
        )?;

        Ok(SwapChainDependentFields {
            swap_chain_container,
            image_views_container,
            color_resource,
            depth_resource,
        })
    }

    /// The swap-chain is a series of framebuffers that are to be presented to the graphics display.
    /// If the display changes size, then we need to recreate it.
    pub fn recreate_swap_chain(&mut self, window_size: &WindowSize) -> Result<()> {
        unsafe {
            self.logical_device.device_wait_idle()?;
            if let Some(render_command) = &mut self.current_render_command {
                render_command.cleanup_swap_chain(&self.logical_device, self.command_pool);
            }
            self.cleanup_swap_chain();
        }

        self.dependent_fields = Self::create_swapchain_dependent_fields(
            &self.instance,
            self.physical_device,
            &self.logical_device,
            &self.queue_indices,
            &self.surface_container,
            &self.command_pool,
            &self.queues,
            window_size,
            self.msaa_samples,
        )?;

        if let Some(render_command) = &mut self.current_render_command {
            render_command.dependent_fields = Self::create_render_command_swap_chain_fields(
                render_command.vertex_shader_file.as_path(),
                render_command.fragment_shader_file.as_path(),
                &render_command.uniform_spec,
                &self.instance,
                self.physical_device,
                &self.logical_device,
                &render_command.texture_image,
                &render_command.vertex_buffer,
                &render_command.index_buffer,
                &self.command_pool,
                render_command.uniform_descriptors,
                &self.dependent_fields,
                self.msaa_samples,
            )?;
        }

        Ok(())
    }

    /// Check to see if we need to handle a resize of the display window.
    /// Whether a resize needs to happen depends on what `location` in draw-frame the resize happened and on the result of
    /// a vulkan operation which may signal that the swapchain is out of date
    fn handle_resize<A>(
        &mut self,
        location: ResizeDetectedLocation,
        result: &VkResult<A>,
        window_size: &WindowSize,
    ) -> Result<bool> {
        let resize_needed = match location {
            ResizeDetectedLocation::InAcquire => match result {
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,
                Err(error) => return Err(VulkanError::from(*error)),
                Ok(_) => false,
            },
            ResizeDetectedLocation::InPresent => match result {
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => true,
                Err(error) => return Err(VulkanError::from(*error)),
                // if a window event signaled that a resize happened then we want to handle the resize after image present
                Ok(_) => self.window_resized,
            },
        };
        let resize_happened = if resize_needed {
            self.window_resized = false;
            self.recreate_swap_chain(window_size)?;
            true
        } else {
            false
        };
        Ok(resize_happened)
    }

    /// Goes through all the vulkan steps needed to render a frame.
    pub fn draw_frame(&mut self, window_size: &WindowSize, apptime: &AppTime) -> Result<()> {
        if self.window_minimized {
            return Ok(());
        }

        if self.current_render_command.is_none() {
            info!("No render command was submitted");
            return Ok(());
        };

        let cpu_gpu_to_wait_for = [self.sync_container.get_in_flight_fence()];
        unsafe {
            self.logical_device
                .wait_for_fences(&cpu_gpu_to_wait_for, true, u64::MAX)?;
        }

        // get an available image from the swapchain
        let timeout = u64::MAX;
        let acquire_result = unsafe {
            self.dependent_fields
                .swap_chain_container
                .swap_chain_loader
                .acquire_next_image(
                    self.dependent_fields.swap_chain_container.swap_chain,
                    timeout,
                    self.sync_container.get_image_available_semaphore(),
                    vk::Fence::null(),
                )
        };
        if self.handle_resize(
            ResizeDetectedLocation::InAcquire,
            &acquire_result,
            window_size,
        )? {
            return Ok(());
        }
        let (available_image_index_u32, _) = acquire_result?;
        let available_image_index = usize::try_from(available_image_index_u32)?;

        // wait on fence to see if image isn't being used already by an in-flight frame
        if self
            .sync_container
            .get_image_in_flight_fence(available_image_index)
            != vk::Fence::null()
        {
            let image_fence = [self
                .sync_container
                .get_image_in_flight_fence(available_image_index)];
            unsafe {
                self.logical_device
                    .wait_for_fences(&image_fence, true, u64::MAX)?;
            }
        }
        // save the fence that will be used for the image used by this in-flight frame
        self.sync_container.set_image_in_flight_fence(
            available_image_index,
            self.sync_container.get_in_flight_fence(),
        );

        // after image is acquired from swap chain we can update the uniform buffer for that swap chain
        let uniform_update_input = UniformUpdateInput {
            swapchain_image_width: self
                .dependent_fields
                .swap_chain_container
                .swap_chain_extent
                .width,
            swapchain_image_height: self
                .dependent_fields
                .swap_chain_container
                .swap_chain_extent
                .height,
            apptime,
        };
        self.update_uniform(uniform_update_input, available_image_index)?;

        // specify that we want to delay the execution of the submit of the command buffer
        // specificially, we want to wait until the wiriting to the color attachment is done on the available image
        let wait_semaphores = [self.sync_container.get_image_available_semaphore()];
        let wait_semaphores_count = u32::try_from(wait_semaphores.len())?;
        let wait_stages: Vec<_> = wait_semaphores
            .iter()
            .map(|_x| vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .collect();

        let command_buffer_ptr = RenderCommand::get_command_buffer_at(
            &self.current_render_command,
            available_image_index,
        )?;

        let signal_semaphores = [self.sync_container.get_render_finished_semaphore()];
        let signal_semaphores_count = u32::try_from(signal_semaphores.len())?;

        let command_buffer_submit_infos = [vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores_count,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: command_buffer_ptr,
            signal_semaphore_count: signal_semaphores_count,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        }];

        let graphics_queue = self.queues.get_graphics_queue()?;
        let cpu_gpu_fence = self.sync_container.get_in_flight_fence();
        unsafe {
            self.logical_device.reset_fences(&[cpu_gpu_fence])?;
            self.logical_device.queue_submit(
                graphics_queue,
                &command_buffer_submit_infos,
                cpu_gpu_fence,
            )?
        }

        // present the image to swap chain
        let swap_chains = [self.dependent_fields.swap_chain_container.swap_chain];
        let swap_chain_count = u32::try_from(swap_chains.len())?;
        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: signal_semaphores_count,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            p_swapchains: swap_chains.as_ptr(),
            swapchain_count: swap_chain_count,
            p_image_indices: &available_image_index_u32,
            ..Default::default()
        };

        let present_queue = self.queues.get_present_queue()?;
        let present_result = unsafe {
            self.dependent_fields
                .swap_chain_container
                .swap_chain_loader
                .queue_present(present_queue, &present_info)
        };

        let _resize_happened = self.handle_resize(
            ResizeDetectedLocation::InPresent,
            &present_result,
            window_size,
        )?;

        self.sync_container.update_frame_counter();

        Ok(())
    }

    /// Block until all operations on queues are done.
    pub fn wait_until_device_idle(&self) -> Result<()> {
        unsafe {
            self.logical_device.device_wait_idle()?;
        }
        Ok(())
    }

    /// This function fills the device memory with the the uniform data.
    /// Uniforms objects are something that only the user of mimic knows about.
    /// As far as mimic is concerned everything around a uniform (its type, its size) is encapsulated inside the uniform_metadata on the rendercommand
    /// However, we still need to update the uniforms and ensure that this happens with the correct synchronization barriers
    pub fn update_uniform(
        &self,
        frame_data_input: UniformUpdateInput,
        swapchain_image_index: usize,
    ) -> Result<()> {
        if let Some(render_command) = &self.current_render_command {
            if swapchain_image_index >= render_command.dependent_fields.uniform_buffers.len() {
                return Err(VulkanError::UniformBufferNotAvailable(
                    swapchain_image_index,
                ));
            }
            memory::fill_uniform_buffer(
                frame_data_input,
                &render_command.uniform_spec,
                &self.logical_device,
                render_command.dependent_fields.uniform_buffers[swapchain_image_index].memory,
            )?;
            Ok(())
        } else {
            Err(VulkanError::RenderCommandNotAvailable)
        }
    }

    /// Create an Ash instance.
    /// Ash is the vulkan rust library that provides the unsafe binding functions to call vulkan API from rust.
    fn create_instance(
        window_title: &str,
        engine_name: &str,
        entry: &ash::Entry,
        validation: &VulkanValidation,
    ) -> Result<ash::Instance> {
        if !(validation.check_validation_layer_support(entry)?) {
            return Err(VulkanError::RequiredValidationLayersUnsupported);
        }

        let app_name = CString::new(window_title).unwrap();
        let engine_name = CString::new(engine_name).unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_version(1, 0, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_version(1, 0, 0),
            api_version: vk::make_api_version(0, 1, 0, 0),
        };

        let extension_names = util::platform::required_extension_names();

        let debug_create_info = VulkanDebug::get_creation_destruction_debug_create_info(validation);
        let debug_create_info_ptr = if let Some(debug_create_info) = debug_create_info {
            (&debug_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT) as *const std::os::raw::c_void
        } else {
            ptr::null()
        };

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: debug_create_info_ptr,
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: validation.get_enabled_layer_count(),
            pp_enabled_layer_names: validation.get_enabled_layer_names(),
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
        };

        let instance: ash::Instance = unsafe { entry.create_instance(&create_info, None)? };

        Ok(instance)
    }

    /// Cleanup all objects that depend on the swap chain
    unsafe fn cleanup_swap_chain(&mut self) {
        std::mem::take(&mut self.dependent_fields.color_resource).drop(&self.logical_device);
        std::mem::take(&mut self.dependent_fields.depth_resource).drop(&self.logical_device);

        for &image_view in &self.dependent_fields.image_views_container.image_views {
            self.logical_device.destroy_image_view(image_view, None);
        }
        self.dependent_fields
            .swap_chain_container
            .swap_chain_loader
            .destroy_swapchain(self.dependent_fields.swap_chain_container.swap_chain, None);
    }
}

impl RenderCommand {
    fn get_command_buffer_at(
        optional_self: &Option<Self>,
        available_image_index: usize,
    ) -> Result<&vk::CommandBuffer> {
        if let Some(render_command) = optional_self {
            if !render_command.dependent_fields.command_buffers.is_empty() {
                Ok(&render_command.dependent_fields.command_buffers[available_image_index])
            } else {
                Err(VulkanError::CommandBufferNotAvailable(
                    available_image_index,
                ))
            }
        } else {
            Err(VulkanError::RenderCommandNotAvailable)
        }
    }

    unsafe fn cleanup_swap_chain(
        &mut self,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
    ) {
        for uniform_buffer in self.dependent_fields.uniform_buffers.iter() {
            logical_device.destroy_buffer(uniform_buffer.buffer, None);
            logical_device.free_memory(uniform_buffer.memory, None);
        }

        for framebuffer in self.dependent_fields.framebuffers.iter() {
            logical_device.destroy_framebuffer(*framebuffer, None);
        }

        logical_device.destroy_pipeline(self.dependent_fields.graphics_pipeline.pipeline, None);
        logical_device.destroy_pipeline_layout(
            self.dependent_fields.graphics_pipeline.pipeline_layout,
            None,
        );

        logical_device
            .destroy_render_pass(self.dependent_fields.graphics_pipeline.render_pass, None);

        // the descriptor sets are cleared automatically when the pool is cleared
        logical_device
            .destroy_descriptor_pool(self.dependent_fields.descriptor_data.descriptor_pool, None);

        logical_device.free_command_buffers(command_pool, &self.dependent_fields.command_buffers);
    }

    unsafe fn cleanup(&mut self, logical_device: &ash::Device, command_pool: vk::CommandPool) {
        self.cleanup_swap_chain(logical_device, command_pool);

        std::mem::take(&mut self.texture_image).cleanup(logical_device);
        std::mem::take(&mut self.index_buffer).cleanup(logical_device);
        std::mem::take(&mut self.vertex_buffer).cleanup(logical_device);

        logical_device.destroy_descriptor_set_layout(self.uniform_descriptors, None);
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        info!("VulkanApp exiting");
        unsafe {
            if let Some(mut render_command) = std::mem::take(&mut self.current_render_command) {
                render_command.cleanup(&self.logical_device, self.command_pool);
            }
            self.cleanup_swap_chain();

            self.sync_container.destroy(&self.logical_device);
            self.logical_device
                .destroy_command_pool(self.command_pool, None);

            self.logical_device.destroy_device(None);
            self.surface_container
                .surface_loader
                .destroy_surface(self.surface_container.surface, None);
            self.debug.destroy_debug_messenger();
            self.instance.destroy_instance(None);
        }
    }
}
//////////////////////// Fns ///////////////////////
fn is_swap_chain_adequate(swap_chain_details: &SwapChainSupportDetails) -> bool {
    !swap_chain_details.formats.is_empty() && !swap_chain_details.present_modes.is_empty()
}

fn is_device_supporting_features(physical_device_featrues: &vk::PhysicalDeviceFeatures) -> bool {
    physical_device_featrues.sampler_anisotropy == vk::TRUE
}
