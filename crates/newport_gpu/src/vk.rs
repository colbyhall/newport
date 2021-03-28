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
use std::slice::{ from_ref, from_raw_parts };
use std::sync::{ RwLock, Mutex };
use std::ffi::CStr;
use std::mem::size_of;

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
    current: Option<usize>,
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
                if it.format == vk::Format::B8G8R8A8_SRGB && it.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
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
                current: None,
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

#[must_use = "this `Receipt` must hold a lifetime until end of scope"]
pub struct Receipt {
    owner: Arc<Device>,

    semaphore: vk::Semaphore,
    fence:     vk::Fence,
}

impl Receipt {
    fn new(owner: Arc<Device>) -> Self {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let semaphore = unsafe{ owner.logical.create_semaphore(&semaphore_create_info, None).unwrap() };

        let fence_create_info = vk::FenceCreateInfo::builder();
        let fence = unsafe{ owner.logical.create_fence(&fence_create_info, None).unwrap() };
        Self{
            owner:     owner,
            semaphore: semaphore,
            fence:     fence
        }
    }
}

impl GenericReceipt for Receipt { }

impl Drop for Receipt {
    fn drop(&mut self) {
        unsafe{
            self.owner.logical.wait_for_fences(&[self.fence], true, 1 << 63).unwrap();
            self.owner.logical.destroy_fence(self.fence, None);
            self.owner.logical.destroy_semaphore(self.semaphore, None);
        }
    }
}

#[derive(Default, Copy, Clone)]
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

    swapchain:   Mutex<Option<Swapchain>>,
    thread_info: Mutex<HashMap<ThreadId, DeviceThreadInfo>>,
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
            
            swapchain:   Mutex::new(None),
            thread_info: Mutex::new(HashMap::new()),
        });

        {
            let mut swapchain = result.swapchain.lock().unwrap();
            *swapchain = Some(Swapchain::new(result.clone()));
        }

        Ok(result)
    }

    fn acquire_backbuffer(&self) -> (Arc<Texture>, Receipt) {
        assert_eq!(self.surface.is_some(), true);

        let mut swapchain = self.swapchain.lock().unwrap();

        let receipt = Receipt::new(swapchain.as_ref().unwrap().backbuffers[0].owner.clone());

        let swapchain_khr = khr::Swapchain::new(&self.owner.instance, &self.logical);
        let mut result = unsafe{ swapchain_khr.acquire_next_image(swapchain.as_ref().unwrap().handle, 1 << 63, receipt.semaphore, vk::Fence::default()) };
        if result.is_err() {
            *swapchain = Some(Swapchain::new(swapchain.as_ref().unwrap().backbuffers[0].owner.clone()));
            result = unsafe{ swapchain_khr.acquire_next_image(swapchain.as_ref().unwrap().handle, 1 << 63, receipt.semaphore, vk::Fence::default()) };
        }
        let (index, _) = result.unwrap();

        swapchain.as_mut().unwrap().current = Some(index as usize);

        (swapchain.as_ref().unwrap().backbuffers[index as usize].clone(), receipt)
    }

    fn submit_graphics(&self, contexts: &[&GraphicsContext], wait_on: &[&Receipt]) -> Receipt {
        let mut buffers = Vec::with_capacity(contexts.len());
        for it in contexts.iter() {
            buffers.push(it.command_buffer);
        }

        let receipt = Receipt::new(contexts[0].owner.clone());

        let mut submit_info = vk::SubmitInfo::builder()
            .command_buffers(&buffers[..])
            .signal_semaphores(from_ref(&receipt.semaphore));
        
        let mut wait_semaphores = Vec::with_capacity(wait_on.len());
        let mut wait_stages = Vec::with_capacity(wait_on.len());
        if wait_on.len() > 0 {

            for it in wait_on.iter() {
                wait_semaphores.push(it.semaphore);
                wait_stages.push(vk::PipelineStageFlags::BOTTOM_OF_PIPE);
            }

            submit_info = submit_info
                .wait_semaphores(&wait_semaphores[..])
                .wait_dst_stage_mask(&wait_stages[..]);
        }

        unsafe{ self.logical.queue_submit(self.graphics_queue.unwrap(), from_ref(&submit_info), receipt.fence).unwrap() };
        
        return receipt;
    }

    fn display(&self, wait_on: &[&Receipt]) {
        assert_eq!(self.surface.is_some(), true);

        let mut swapchain = self.swapchain.lock().unwrap();
        let swapchain_khr = khr::Swapchain::new(&self.owner.instance, &self.logical);

        let index = swapchain.as_ref().unwrap().current.expect("Backbuffer was not acquired") as u32;

        let mut present_info = vk::PresentInfoKHR::builder()
            .swapchains(from_ref(&swapchain.as_ref().unwrap().handle))
            .image_indices(from_ref(&index));
        
        let mut wait_semaphores = Vec::with_capacity(wait_on.len());
        if wait_on.len() > 0 {
            for it in wait_on.iter() {
                wait_semaphores.push(it.semaphore);
            }

            present_info = present_info
                .wait_semaphores(&wait_semaphores[..]);
        }

        let result = unsafe{ swapchain_khr.queue_present(self.presentation_queue.unwrap(), &present_info) };
        if result.is_err() {
            *swapchain = Some(Swapchain::new(swapchain.as_ref().unwrap().backbuffers[0].owner.clone()));
        }
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

impl Drop for RenderPass {
    fn drop(&mut self) {
        todo!()
    }
}

fn shader_variant_to_shader_stage(variant: ShaderVariant) -> vk::ShaderStageFlags {
    match variant {
        ShaderVariant::Vertex => vk::ShaderStageFlags::VERTEX,
        ShaderVariant::Pixel => vk::ShaderStageFlags::FRAGMENT,
    }
}

pub struct Shader {
    owner:   Arc<Device>,

    variant: ShaderVariant,
    module:  vk::ShaderModule,
}

impl GenericShader for Shader {
    fn new(owner: Arc<Device>, contents: Vec<u8>, variant: ShaderVariant) -> Result<Arc<Shader>, ()> {
        let contents = unsafe{ from_raw_parts(contents.as_ptr() as *const u32, contents.len() / 4) };

        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(contents);

        let shader = unsafe{ owner.logical.create_shader_module(&create_info, None) };
        if shader.is_err() {
            return Err(());
        }
        let shader = shader.unwrap();

        Ok(Arc::new(Shader{
            owner: owner,

            variant: variant,
            module:  shader,
        }))
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        todo!();
    }
}

pub struct Pipeline {
    owner: Arc<Device>,

    handle: vk::Pipeline,
    layout: vk::PipelineLayout,

    desc: PipelineDescription,
}

impl GenericPipeline for Pipeline {
    fn new(owner: Arc<Device>, desc: PipelineDescription) -> Result<Arc<Pipeline>, ()> {
        match desc {
            PipelineDescription::Graphics(desc) => {
                assert!(desc.shaders.len() > 0);

                // Create all the shader staage info for pipeline
                let mut shader_stages = Vec::with_capacity(desc.shaders.len());
                for it in desc.shaders.iter() {
                    let stage = shader_variant_to_shader_stage(it.variant);

                    let stage_info = unsafe{ 
                        vk::PipelineShaderStageCreateInfo::builder()
                            .stage(stage)
                            .module(it.module)
                            .name(CStr::from_ptr(b"main\0".as_ptr() as *const i8))
                    };
                    
                    shader_stages.push(stage_info.build());
                }

                let mut stride = 0;
                for it in desc.vertex_attributes.iter() {
                    stride += it.size();
                }

                // Setup the vertex attributes
                let binding = vk::VertexInputBindingDescription::builder()
                    .binding(0)
                    .stride(stride as u32)
                    .input_rate(vk::VertexInputRate::VERTEX);

                let mut attributes = Vec::with_capacity(desc.vertex_attributes.len());
                let mut offset = 0;
                for (index, it) in desc.vertex_attributes.iter().enumerate() {
                    let format = match it {
                        VertexAttribute::Int32   => vk::Format::R32_SINT,
                        VertexAttribute::Float32 => vk::Format::R32_SFLOAT,
                        VertexAttribute::Vector2 => vk::Format::R32G32_SFLOAT,
                        VertexAttribute::Vector3 => vk::Format::R32G32B32_SFLOAT,
                        VertexAttribute::Vector4 => vk::Format::R32G32B32A32_SFLOAT,
                        VertexAttribute::Color   => vk::Format::R32G32B32A32_SFLOAT,
                    };

                    let attr = vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location(index as u32)
                        .offset(offset as u32)
                        .format(format);

                    offset += it.size();

                    attributes.push(attr.build());
                }

                let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
                    .vertex_binding_descriptions(from_ref(&binding))
                    .vertex_attribute_descriptions(&attributes[..]);

                let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
                    .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

                let viewport = vk::Viewport::builder()
                    .width(100.0)
                    .height(100.0)
                    .max_depth(1.0);
                let scissor = vk::Rect2D::builder()
                    .extent(vk::Extent2D::builder()
                        .width(100)
                        .height(100)
                        .build()
                    );

                let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                    .viewports(from_ref(&viewport))
                    .scissors(from_ref(&scissor));
                
                let polygon_mode = match desc.draw_mode {
                    DrawMode::Fill  => vk::PolygonMode::FILL,
                    DrawMode::Line  => vk::PolygonMode::LINE,
                    DrawMode::Point => vk::PolygonMode::POINT,
                };

                let mut cull = vk::CullModeFlags::NONE;
                if desc.cull_mode.contains(CullMode::FRONT) {
                    cull |= vk::CullModeFlags::FRONT;
                }
                if desc.cull_mode.contains(CullMode::BACK) {
                    cull |= vk::CullModeFlags::BACK;
                }

                // NOTE: Depth Testing goes around here somewhere
                let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::builder()
                    .polygon_mode(polygon_mode)
                    .cull_mode(cull)
                    .front_face(vk::FrontFace::CLOCKWISE)
                    .line_width(desc.line_width);

                let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
                    .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                    .min_sample_shading(1.0);

                // Setting up blending and converting data types
                fn blend_factor(fc: BlendFactor) -> vk::BlendFactor {
                    match fc {
                        BlendFactor::Zero               => return vk::BlendFactor::ZERO,
                        BlendFactor::One                => return vk::BlendFactor::ONE,
                        BlendFactor::SrcColor           => return vk::BlendFactor::SRC_COLOR,
                        BlendFactor::OneMinusSrcColor   => return vk::BlendFactor::ONE_MINUS_SRC_COLOR,
                        BlendFactor::DstColor           => return vk::BlendFactor::DST_COLOR,
                        BlendFactor::OneMinusDstColor   => return vk::BlendFactor::ONE_MINUS_DST_COLOR,
                        BlendFactor::SrcAlpha           => return vk::BlendFactor::SRC_ALPHA,
                        BlendFactor::OneMinusSrcAlpha   => return vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                    }
                }

                fn blend_op(a: BlendOp) -> vk::BlendOp{
                    match a {
                        BlendOp::Add              => vk::BlendOp::ADD,
                        BlendOp::Subtract         => vk::BlendOp::SUBTRACT,
                        BlendOp::ReverseSubtract  => vk::BlendOp::REVERSE_SUBTRACT,
                        BlendOp::Min              => vk::BlendOp::MIN,
                        BlendOp::Max              => vk::BlendOp::MAX,
                    }
                }

                let mut color_write_mask = vk::ColorComponentFlags::default();
                if desc.color_mask.contains(ColorMask::RED) {
                    color_write_mask |= vk::ColorComponentFlags::R;
                }
                if desc.color_mask.contains(ColorMask::GREEN) {
                    color_write_mask |= vk::ColorComponentFlags::G;
                }
                if desc.color_mask.contains(ColorMask::BLUE) {
                    color_write_mask |= vk::ColorComponentFlags::B;
                }
                if desc.color_mask.contains(ColorMask::ALPHA) {
                    color_write_mask |= vk::ColorComponentFlags::A;
                }

                let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(desc.blend_enabled)
                    .src_color_blend_factor(blend_factor(desc.src_color_blend_factor))
                    .dst_color_blend_factor(blend_factor(desc.dst_color_blend_factor))
                    .color_blend_op(blend_op(desc.color_blend_op))
                    .src_alpha_blend_factor(blend_factor(desc.src_alpha_blend_factor))
                    .dst_alpha_blend_factor(blend_factor(desc.dst_alpha_blend_factor))
                    .alpha_blend_op(blend_op(desc.alpha_blend_op))
                    .color_write_mask(color_write_mask);

                let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                    .logic_op(vk::LogicOp::COPY)
                    .attachments(from_ref(&color_blend_attachment));
                
                let dynamic_states = [
                    vk::DynamicState::VIEWPORT,
                    vk::DynamicState::SCISSOR,
                ];

                let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
                    .dynamic_states(&dynamic_states);
                
                let mut pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder(); // TODO: Do bindless descriptor layout
                
                // assert(push_constant_size <= 128); // Min push contsant size
                let range = vk::PushConstantRange::builder()
                    .size(desc.push_constant_size as u32)
                    .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS);

                if desc.push_constant_size > 0 {
                    pipeline_layout_info = pipeline_layout_info
                        .push_constant_ranges(from_ref(&range));
                }

                let layout = unsafe{ owner.logical.create_pipeline_layout(&pipeline_layout_info, None) };
                if layout.is_err() {
                    return Err(());
                }
                let layout = layout.unwrap();

                let create_info = vk::GraphicsPipelineCreateInfo::builder()
                    .stages(&shader_stages[..])
                    .vertex_input_state(&vertex_input_state)
                    .input_assembly_state(&input_assembly_state)
                    .viewport_state(&viewport_state)
                    .rasterization_state(&rasterizer_state)
                    .multisample_state(&multisample_state)
                    .color_blend_state(&color_blend_state)
                    .dynamic_state(&dynamic_state)
                    .layout(layout)
                    .render_pass(desc.render_pass.handle)
                    .base_pipeline_index(-1);

                let handle = unsafe{ owner.logical.create_graphics_pipelines(vk::PipelineCache::default(), from_ref(&create_info), None) };
                if handle.is_err() {
                    return Err(());
                }
                let handle = handle.unwrap();

                Ok(Arc::new(Pipeline {
                    owner: owner,
                    
                    handle: handle[0],
                    layout: layout,

                    desc: PipelineDescription::Graphics(desc),
                }))
            }
        }
    }
}

pub struct GraphicsContext<'a> {
    owner: Arc<Device>,

    command_buffer: vk::CommandBuffer,
    framebuffers:   Vec<vk::Framebuffer>,

    pipelines:   Vec<Arc<Pipeline>>,
    attachments: Vec<Arc<Texture>>,

    current_render_pass: Option<&'a Arc<RenderPass>>,
    current_scissor: Option<Rect>
}

fn layout_to_image_layout(layout: Layout) -> vk::ImageLayout {
    match layout {
        Layout::Undefined       => vk::ImageLayout::UNDEFINED,
        Layout::General         => vk::ImageLayout::GENERAL,
        Layout::ColorAttachment => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        Layout::DepthAttachment => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        Layout::TransferSrc     => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        Layout::TransferDst     => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        Layout::ShaderReadOnly  => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        Layout::Present         => vk::ImageLayout::PRESENT_SRC_KHR,
    }
}

impl<'a> GenericContext for GraphicsContext<'a> {
    fn begin(&mut self) {
        unsafe{ self.owner.logical.reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::default()).unwrap() };
        
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe{ self.owner.logical.begin_command_buffer(self.command_buffer, &begin_info).unwrap() };
    }

    fn end(&mut self) {
        unsafe{ self.owner.logical.end_command_buffer(self.command_buffer).unwrap() };
    }

    fn copy_buffer_to_texture(&mut self, dst: Arc<Texture>, _src: Arc<Buffer>) {
        let subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .layer_count(1);
        
        let extent = vk::Extent3D::builder()
            .width(dst.width)
            .height(dst.height)
            .depth(dst.depth);

        let _region = vk::BufferImageCopy::builder()
            .image_subresource(subresource.build())
            .image_extent(extent.build());
        
        // self.owner.logical.cmd_copy_buffer_to_image(self.command_buffer, src.handle, dst.image)
        todo!();
    }

    fn resource_barrier_texture(&mut self, _texture: Arc<Texture>, _old_layout: Layout, _new_layout: Layout) {
        todo!();
    }
}

impl<'a> GenericGraphicsContext<'a> for GraphicsContext<'a> {
    fn new(owner: Arc<Device>) -> Result<GraphicsContext<'a>, ()> {
        let handle = {
            let mut thread_infos = owner.thread_info.lock().unwrap();
            let thread_id = std::thread::current().id();
    
            let mut thread_info = thread_infos.get_mut(&thread_id);
            if thread_info.is_none() {
                thread_infos.insert(thread_id, DeviceThreadInfo::default());
                thread_info = thread_infos.get_mut(&thread_id)
            }
            let thread_info = thread_info.unwrap();
    
            if thread_info.graphics_pool == vk::CommandPool::default() {
                let create_info = vk::CommandPoolCreateInfo::builder()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(owner.graphics_family_index.unwrap());
    
                thread_info.graphics_pool = unsafe{ owner.logical.create_command_pool(&create_info, None).unwrap() };
            }
    
            let alloc_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(thread_info.graphics_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);
            
            let handle = unsafe{ owner.logical.allocate_command_buffers(&alloc_info) };
            if handle.is_err() {
                return Err(());
            }
            handle.unwrap()[0]
        };

        Ok(GraphicsContext{
            owner: owner,

            command_buffer: handle,
            framebuffers:   Vec::new(),

            pipelines:   Vec::new(),
            attachments: Vec::new(),

            current_render_pass: None,
            current_scissor: None,
        })
    }

    fn begin_render_pass(&mut self, render_pass: &'a Arc<RenderPass>, attachments: &[Arc<Texture>]) {
        let extent = vk::Extent2D::builder()
            .width(attachments[0].width)
            .height(attachments[0].height)
            .build();
        
        self.current_render_pass = Some(render_pass);
        for it in attachments.iter() {
            self.attachments.push(it.clone());
        }

        // Make the framebuffer
        let mut views = Vec::with_capacity(attachments.len());
        for it in attachments.iter() {
            views.push(it.view);
        }

        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass.handle)
            .attachments(&views[..])
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer = unsafe{ self.owner.logical.create_framebuffer(&create_info, None).unwrap() };
        self.framebuffers.push(framebuffer);

        let render_area = vk::Rect2D::builder()
            .extent(extent);

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.handle)
            .framebuffer(framebuffer)
            .render_area(render_area.build());
        
        unsafe{ self.owner.logical.cmd_begin_render_pass(self.command_buffer, &begin_info, vk::SubpassContents::INLINE) };
    }

    fn end_render_pass(&mut self) {
        unsafe{ self.owner.logical.cmd_end_render_pass(self.command_buffer) };
        self.current_render_pass = None;
        self.current_scissor = None;
    }

    fn bind_scissor(&mut self, scissor: Option<Rect>) {
        self.current_scissor = scissor;
    }

    fn bind_pipeline(&mut self, pipeline: Arc<Pipeline>) {
        unsafe {
            self.owner.logical.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.handle);
            // self.owner.logical.cmd_bind_descriptor_sets(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, ...);
            // TODO: Bindless

            let viewport = vk::Viewport::builder()
                .width(self.attachments.last().unwrap().width as f32)
                .height(self.attachments.last().unwrap().height as f32)
                .max_depth(1.0);
            self.owner.logical.cmd_set_viewport(self.command_buffer, 0, from_ref(&viewport));

            if self.current_scissor.is_some() {
                let scissor = self.current_scissor.unwrap();

                let size = scissor.size();
                let rect = vk::Rect2D::builder()
                    .offset(
                        vk::Offset2D::builder()
                            .x(scissor.min.x as i32)
                            .y(scissor.min.y as i32)
                            .build()
                        )
                    .extent(
                        vk::Extent2D::builder()
                            .width(size.x as u32)
                            .height(size.y as u32)
                            .build()
                        );
                
                self.owner.logical.cmd_set_scissor(self.command_buffer, 0, from_ref(&rect));
            } else {
                let rect = vk::Rect2D::builder()
                    .extent(
                        vk::Extent2D::builder()
                            .width(viewport.x as u32)
                            .height(viewport.y as u32)
                            .build()
                        );
                self.owner.logical.cmd_set_scissor(self.command_buffer, 0, from_ref(&rect));
            }
        }

        self.pipelines.push(pipeline);
    }

    fn bind_vertex_buffer(&mut self, buffer: Arc<Buffer>) {
        let offset = 0;
        unsafe{ self.owner.logical.cmd_bind_vertex_buffers(self.command_buffer, 0, from_ref(&buffer.handle), from_ref(&offset)) };
    }

    fn draw(&mut self, vertex_count: usize, first_vertex: usize) {
        unsafe{ self.owner.logical.cmd_draw(self.command_buffer, vertex_count as u32, 1, first_vertex as u32, 0) };
    }

    fn clear(&mut self, _color: Color, _attachments: &[Arc<Texture>]) {
        todo!();
    }

    fn push_constants<T>(&mut self, t: T) {
        let pipeline = &self.pipelines.last().unwrap();

        let desc = match &pipeline.desc {
            PipelineDescription::Graphics(desc) => desc
        };
        assert_eq!(size_of::<T>(), desc.push_constant_size);

        unsafe{ 
            self.owner.logical.cmd_push_constants(
                self.command_buffer, 
                pipeline.layout, 
                vk::ShaderStageFlags::ALL_GRAPHICS, 
                0, 
                from_raw_parts(&t as *const T as *const u8, size_of::<T>()),
            );
        }
    }
}

impl<'a> Drop for GraphicsContext<'a> {
    fn drop(&mut self) {
        todo!();
    }
}