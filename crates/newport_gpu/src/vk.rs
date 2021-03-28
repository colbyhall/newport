// #![allow(unused_variables)]
#![allow(dead_code)]
use crate::*;

#[cfg(target_os = "windows")]
use newport_os::win32;

use newport_core::containers::HashMap;

use std::thread::ThreadId;

use ash::{ vk, extensions::khr };
use ash::version::{ EntryV1_0, InstanceV1_0, InstanceV1_1, DeviceV1_0 };

use std::ptr::{ null_mut, copy_nonoverlapping };
use std::slice::from_ref;
use std::sync::{ RwLock, Mutex };

const ENABLED_LAYER_NAMES: [*const i8; 1] = [
    b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8
];

pub struct Instance {
    entry:    ash::Entry, // We need to keep this around for post_init
    instance: ash::Instance,
}

impl GenericInstance for Instance {
    fn new() -> Result<Arc<Self>, InstanceCreateError> {
        let entry = unsafe{ 
            let entry = ash::Entry::new();
            if entry.is_err() {
                return Err(InstanceCreateError::FailedToLoadLibrary);
            }
            entry.unwrap()
        };

        let app_info = vk::ApplicationInfo{
            api_version: vk::make_version(1, 0, 0),
            ..Default::default()
        };

        #[cfg(target_os = "windows")]
        let enabled_extension_names = [
            b"VK_KHR_surface\0".as_ptr() as *const i8,
            b"VK_KHR_win32_surface\0".as_ptr() as *const i8
        ];

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names)
            .enabled_layer_names(&ENABLED_LAYER_NAMES);
        let instance = unsafe{ 
            let instance = entry.create_instance(&create_info, None);
            if instance.is_err() {
                let err = instance.err().unwrap();
                match err {
                    ash::InstanceError::LoadError(_err) => return Err(InstanceCreateError::FailedToLoadLibrary),
                    ash::InstanceError::VkError(err) => {
                        match err {
                            vk::Result::ERROR_INCOMPATIBLE_DRIVER => return Err(InstanceCreateError::IncompatibleDriver),
                            _ => return Err(InstanceCreateError::Unknown),
                        }
                    }
                }
            }
            instance.unwrap()
        };

        Ok(Arc::new(Self {
            entry:    entry,
            instance: instance,
        }))
    }
}

struct Swapchain {
    handle: vk::SwapchainKHR,
    extent: vk::Extent2D,
    format: Format,

    backbuffers: Vec<Arc<Texture>>,
}

impl Swapchain {
    fn new(device: Arc<Device>) -> Self {
        assert_eq!(device.surface.is_some(), true);

        let swapchain_khr = khr::Swapchain::new(&device.owner.instance, &device.logical);
        let surface_khr = khr::Surface::new(&device.owner.entry, &device.owner.instance);

        unsafe{ 
            device.logical.device_wait_idle().unwrap();

            let capabilities = surface_khr.get_physical_device_surface_capabilities(device.physical, device.surface.unwrap()).unwrap();
            let formats = surface_khr.get_physical_device_surface_formats(device.physical, device.surface.unwrap()).unwrap();

            let mut selected_format = None;
            for it in formats.iter() {
                if it.format == vk::Format::B8G8R8A8_SINT && it.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                    selected_format = Some(it);
                    break;
                }
            }
            let selected_format = selected_format.unwrap();

            let mut queue_family_indices = Vec::with_capacity(2);
            if device.graphics_family_index.is_some() {
                queue_family_indices.push(device.graphics_family_index.unwrap());
            }
            if device.surface_family_index.is_some() {
                queue_family_indices.push(device.surface_family_index.unwrap());
            }

            let create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(device.surface.unwrap())
                .min_image_count(capabilities.min_image_count)
                .image_format(selected_format.format)
                .image_color_space(selected_format.color_space)
                .image_extent(capabilities.current_extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices[..])
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true);
            
            let handle = swapchain_khr.create_swapchain(&create_info, None).unwrap();

            let images = swapchain_khr.get_swapchain_images(handle).unwrap();
            
            let mut backbuffers = Vec::with_capacity(images.len());
            for it in images.iter() {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(*it)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(selected_format.format)
                    .components(vk::ComponentMapping{
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange{
                        aspect_mask:      vk::ImageAspectFlags::COLOR,
                        base_mip_level:   0,
                        level_count:      1,
                        base_array_layer: 0,
                        layer_count:      1
                    });
                
                let view = device.logical.create_image_view(&create_info, None).unwrap();

                backbuffers.push(Arc::new(Texture{
                    owner: device.clone(),
                    view:  view,
                    
                    memory_type: MemoryType::HostVisible,
                    usage:       TextureUsage::SWAPCHAIN,
                    format:      Format::BGR_U8_SRGB,

                    width:  capabilities.current_extent.width,
                    height: capabilities.current_extent.height,
                    depth:  1
                }));
            }

            Self {
                handle: handle,
                extent: capabilities.current_extent,
                format: Format::BGR_U8_SRGB,
                
                backbuffers: backbuffers,
            }
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe { 
            let swapchain_khr = khr::Swapchain::new(&self.backbuffers[0].owner.owner.instance, &self.backbuffers[0].owner.logical);
            swapchain_khr.destroy_swapchain(self.handle, None);
        }
    }
}

struct DeviceThreadInfo {
    graphics_pool: vk::CommandPool,
    compute_pool:  vk::CommandPool,
    transfer_pool: vk::CommandPool,
}

#[derive(Default, Copy, Clone)]
struct DeviceAllocation {
    memory: vk::DeviceMemory,
    offset: vk::DeviceSize,
    size:   vk::DeviceSize,
}

#[allow(dead_code)]
pub struct Device {
    owner:    Arc<Instance>,

    logical:  ash::Device,
    physical: vk::PhysicalDevice,

    graphics_queue:     Option<vk::Queue>,
    presentation_queue: Option<vk::Queue>,

    graphics_family_index:  Option<u32>,
    surface_family_index:   Option<u32>,

    #[cfg(target_os = "windows")]
    surface: Option<vk::SurfaceKHR>,

    swapchain: Mutex<Option<Swapchain>>,
    thread_info: RwLock<HashMap<ThreadId, DeviceThreadInfo>>,
}

impl Device {
    fn allocate_memory(&self, requirements: vk::MemoryRequirements, memory_type: MemoryType) -> Result<DeviceAllocation, ()> {
        let property_flag = match memory_type {
            MemoryType::DeviceLocal => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            MemoryType::HostVisible => vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE
        };
        unsafe {
            let properties = self.owner.instance.get_physical_device_memory_properties(self.physical);

            let mut index = None;
            for i in 0..properties.memory_type_count {
                let mut can_use = (requirements.memory_type_bits & (1 << i)) != 0;
                can_use &= properties.memory_types[i as usize].property_flags & property_flag != vk::MemoryPropertyFlags::empty();

                if can_use {
                    index = Some(i);
                    break;
                }
            }
            let index = index.unwrap();

            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(requirements.size)
                .memory_type_index(index);
            
            let memory = self.logical.allocate_memory(&alloc_info, None);
            if memory.is_err() {
                return Err(());
            }

            Ok(DeviceAllocation{
                memory: memory.unwrap(),
                offset: 0,
                size:   requirements.size,
            })
        }
    }

    fn free_memory(&self, allocation: DeviceAllocation) {
        unsafe {
            self.logical.free_memory(allocation.memory, None);
        }
    }
}

impl GenericDevice for Device {
    // TODO: Custom allocation logic
    fn new(instance: Arc<Instance>, window: Option<WindowHandle>) -> Result<Arc<Self>, DeviceCreateError> {
        // Find a physical device based off of some parameters
        let physical_device;
        unsafe {
            let physical_devices = instance.instance.enumerate_physical_devices();
            if physical_devices.is_err() {
                return Err(DeviceCreateError::NoValidPhysicalDevice);
            }
            let physical_devices = physical_devices.unwrap();

            let mut selected_device = None;
            for it in physical_devices.iter() {
                let properties = instance.instance.get_physical_device_properties(*it);
                let features = instance.instance.get_physical_device_features(*it);

                // Find extensions to do bindless
                let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::default();

                let mut device_features = vk::PhysicalDeviceFeatures2::default();
                device_features.p_next = &mut indexing_features as *mut vk::PhysicalDeviceDescriptorIndexingFeatures as *mut std::ffi::c_void;
                instance.instance.get_physical_device_features2(*it, &mut device_features);

                // TODO: Maybe do more checking with features we actually will need like KHR Swapchain support?
                //  also maybe take something in from the builder
                let mut is_acceptable = true;
                is_acceptable &= properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU && features.geometry_shader == 1;
                is_acceptable &= indexing_features.descriptor_binding_partially_bound == 1 && indexing_features.runtime_descriptor_array == 1;

                if is_acceptable { selected_device = Some(*it); }
            }

            if selected_device.is_none() {
                return Err(DeviceCreateError::NoValidPhysicalDevice);
            }

            physical_device = selected_device.unwrap();
        }

        // Create the surface if the builder provided one
        #[cfg(target_os = "windows")]
        let surface;
        unsafe {
            if window.is_some() {
                let surface_khr = khr::Win32Surface::new(&instance.entry, &instance.instance);
                let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
                    .hinstance(win32::GetModuleHandleA(null_mut()))
                    .hwnd(window.unwrap());
                    
                surface = Some(surface_khr.create_win32_surface(&create_info, None).unwrap());
            } else {
                surface = None;
            };
        }

        // Find the proper queue family indices 
        let mut graphics_family_index = None;
        let mut surface_family_index = None;
        unsafe {           
            let queue_family_properties = instance.instance.get_physical_device_queue_family_properties(physical_device);
            for (index, it) in queue_family_properties.iter().enumerate() {
                if it.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_family_index = Some(index as u32);
                }

                if window.is_some() {
                    let surface_khr = khr::Surface::new(&instance.entry, &instance.instance);
                    let present_support = surface_khr.get_physical_device_surface_support(physical_device, index as u32, surface.unwrap()).unwrap();
                    if present_support {
                        surface_family_index = Some(index as u32);
                    }
                }
            }
        }

        let queue_family_indices = [
            graphics_family_index,
            surface_family_index,
        ];

        // Create the logical device and the queues
        let logical_device;
        let graphics_queue;
        let presentation_queue;
        unsafe {
            // TODO: Use a custom linear or temp allocator later on when thats created
            let mut queue_create_infos = Vec::new();
            
            let queue_priorities = [0.0];
            for it in queue_family_indices.iter() {
                let create_info = vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(it.unwrap())
                    .queue_priorities(&queue_priorities);

                queue_create_infos.push(create_info.build());
            }

            let device_features = vk::PhysicalDeviceFeatures::default();
            let extensions = [
                b"VK_KHR_swapchain\0".as_ptr() as *const i8
            ];
            let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
                .descriptor_binding_partially_bound(true)
                .runtime_descriptor_array(true);

            let create_info = vk::DeviceCreateInfo::builder()
                .push_next(&mut indexing_features)
                .queue_create_infos(&queue_create_infos[..])
                .enabled_layer_names(&ENABLED_LAYER_NAMES)
                .enabled_extension_names(&extensions)
                .enabled_features(&device_features);

            logical_device = instance.instance.create_device(physical_device, &create_info, None).unwrap();
            graphics_queue = Some(logical_device.get_device_queue(graphics_family_index.unwrap(), 0));

            if surface_family_index.is_some() {
                presentation_queue = Some(logical_device.get_device_queue(surface_family_index.unwrap(), 0));
            } else {
                presentation_queue = None;
            }
        }

        let result = Arc::new(Device{
            owner:    instance,

            logical:  logical_device,
            physical: physical_device,

            graphics_queue:     graphics_queue,
            presentation_queue: presentation_queue,

            graphics_family_index: graphics_family_index,
            surface_family_index:  surface_family_index,

            surface:   surface,
            
            swapchain: Mutex::new(None),
            thread_info: RwLock::new(HashMap::new()),
        });

        {
            let mut swapchain = result.swapchain.lock().unwrap();
            *swapchain = Some(Swapchain::new(result.clone()));
        }

        Ok(result)
    }
}

pub struct Buffer {
    owner:  Arc<Device>,

    handle: vk::Buffer,
    memory: RwLock<DeviceAllocation>,

    usage:  BufferUsage,
}

impl GenericBuffer for Buffer {
    fn new(device: Arc<Device>, usage: BufferUsage, memory: MemoryType, size: usize) -> Result<Arc<Buffer>, ResourceCreateError> {
        let mut vk_usage = vk::BufferUsageFlags::default();
        if usage.contains(BufferUsage::TRANSFER_SRC) {
            vk_usage |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if usage.contains(BufferUsage::TRANSFER_DST) {
            vk_usage |= vk::BufferUsageFlags::TRANSFER_DST;
        }
        if usage.contains(BufferUsage::VERTEX) {
            vk_usage |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if usage.contains(BufferUsage::INDEX) {
            vk_usage |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if usage.contains(BufferUsage::CONSTANTS) {
            vk_usage |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }

        unsafe {
            let create_info = vk::BufferCreateInfo::builder()
                .size(size as vk::DeviceSize)
                .usage(vk_usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE); // TODO: Look into this more
    
            let handle = device.logical.create_buffer(&create_info, None).unwrap();
    
            // Allocate memory for buffer
            let memory = device.allocate_memory(device.logical.get_buffer_memory_requirements(handle), memory);
            if memory.is_err() {
                return Err(ResourceCreateError::OutOfMemory);
            }
            let memory = memory.unwrap();
            device.logical.bind_buffer_memory(handle, memory.memory, 0).unwrap();

            Ok(Arc::new(Buffer{
                owner: device,

                handle: handle,
                memory: RwLock::new(memory),

                usage:  usage,
            }))
        }
    }

    fn copy_to<T>(&self, data: Vec<T>) {
        let memory = self.memory.write().unwrap();

        unsafe {
            let ptr = self.owner.logical.map_memory(memory.memory, memory.offset, memory.size, vk::MemoryMapFlags::empty()).unwrap();
            copy_nonoverlapping(data.as_ptr() as *const u8, ptr as *mut u8, memory.size as usize);
            self.owner.logical.unmap_memory(memory.memory);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.owner.logical.destroy_buffer(self.handle, None);
        }

        let memory = self.memory.write().unwrap();
        self.owner.free_memory(*memory);
    }
}

fn vk_format(format: Format) -> vk::Format {
    match format {
        Format::Undefined    => vk::Format::UNDEFINED,
        Format::RGB_U8       => vk::Format::R8G8B8_UINT,
        Format::RGB_U8_SRGB  => vk::Format::R8G8B8_SRGB,
        Format::RGBA_U8      => vk::Format::R8G8B8A8_UINT,
        Format::RGBA_U8_SRGB => vk::Format::R8G8B8A8_SRGB,
        Format::RGBA_F16     => vk::Format::R16G16B16A16_SFLOAT,
        Format::BGR_U8_SRGB  => vk::Format::B8G8R8A8_SRGB
    }
}

pub struct Texture {
    owner:   Arc<Device>,

    // image:   vk::Image,
    view:    vk::ImageView,
    // sampler: vk::Sampler,
    // memory:  vk::DeviceMemory,

    memory_type: MemoryType,

    usage:   TextureUsage,
    format:  Format,

    width:  u32,
    height: u32,
    depth:  u32,
}


impl GenericTexture for Texture {
    fn owner(&self) -> &Arc<Device> { &self.owner }
    fn memory_type(&self) -> MemoryType { self.memory_type }
    fn usage(&self) -> TextureUsage { self.usage }
    fn format(&self) -> Format { self.format }
    fn width(&self) -> u32 { self.width }
    fn height(&self) -> u32 { self.height }
    fn depth(&self) -> u32 { self.depth }
}

impl Drop for Texture {
    fn drop(&mut self) {
        if self.usage.contains(TextureUsage::SWAPCHAIN) {
            unsafe {
                self.owner.logical.destroy_image_view(self.view, None);
            }
        } else {
            todo!();
        }
    }
}

pub struct RenderPass {
    owner: Arc<Device>,

    handle: vk::RenderPass,

    colors: Vec<Format>,
    depth:  Option<Format,>
}

impl GenericRenderPass for RenderPass {
    fn new(owner: Arc<Device>, colors: Vec<Format>, depth: Option<Format>) -> Result<Arc<RenderPass>, ()> {
        let mut color_refs = Vec::with_capacity(colors.len());
        
        let num_attachments = {
            let num = colors.len();
            if depth.is_some() {
                num + 1
            } else {
                num
            }
        };

        let mut attachments = Vec::with_capacity(num_attachments);

        for (index, it) in colors.iter().enumerate() {
            let format = vk_format(*it);

            let attachment = vk::AttachmentDescription::builder()
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            attachments.push(attachment.build());

            let the_ref = vk::AttachmentReference::builder()
                .attachment(index as u32)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            color_refs.push(the_ref.build());
        }

        // Currently we're only going to support 1 subpass as no other API has subpasses
        let mut subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs[..]);

        let depth_refs = if depth.is_some() {
            let depth = depth.unwrap();
            let format = vk_format(depth);

            let attachment = vk::AttachmentDescription::builder()
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            attachments.push(attachment.build());
            
            let the_ref = vk::AttachmentReference::builder()
                .attachment(num_attachments as u32)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
                
            Some(the_ref.build())
        } else {
            None
        };

        if depth_refs.is_some() {
            subpass = subpass.depth_stencil_attachment(depth_refs.as_ref().unwrap());
        }

        let mut stage_mask = vk::PipelineStageFlags::empty();
        let mut access_mask = vk::AccessFlags::empty();

        if colors.len() > 0 {
            stage_mask |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
            access_mask |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        }

        if depth.is_some() {
            stage_mask |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
            access_mask |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
        }

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(stage_mask)
            .dst_stage_mask(stage_mask)
            .dst_access_mask(access_mask);

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments[..])
            .subpasses(from_ref(&subpass))
            .dependencies(from_ref(&dependency));

        unsafe {
            let handle = owner.logical.create_render_pass(&create_info, None);
            if handle.is_err() {
                return Err(());
            }
            let handle = handle.unwrap();

            Ok(Arc::new(RenderPass{
                owner: owner,
                handle: handle,
                colors: colors,
                depth:  depth,
            }))
        }
    }
}