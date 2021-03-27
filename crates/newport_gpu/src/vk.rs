use crate::*;

use newport_engine::*;
use newport_asset::AssetManager;

#[cfg(target_os = "windows")]
use newport_os::win32;

use ash::{ vk, Entry, Instance, extensions::khr };
use ash::version::{ EntryV1_0, InstanceV1_0, InstanceV1_1, DeviceV1_0 };

use std::ptr::null_mut;

const ENABLED_LAYER_NAMES: [*const i8; 1] = [
    b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8
];

pub struct VulkanGPU {
    entry:    Entry, // We need to keep this around for post_init
    instance: Instance,
}

impl GPU for VulkanGPU {
    fn new_device(&self, builder: DeviceBuilder) -> Result<Box<dyn Device>, DeviceCreateError> {
        // Find a physical device based off of some parameters
        let physical_device;
        unsafe {
            let physical_devices = self.instance.enumerate_physical_devices();
            if physical_devices.is_err() {
                return Err(DeviceCreateError::NoValidPhysicalDevice);
            }
            let physical_devices = physical_devices.unwrap();

            let mut selected_device = None;
            for it in physical_devices.iter() {
                let properties = self.instance.get_physical_device_properties(*it);
                let features = self.instance.get_physical_device_features(*it);

                // Find extensions to do bindless
                let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::default();

                let mut device_features = vk::PhysicalDeviceFeatures2::default();
                device_features.p_next = &mut indexing_features as *mut vk::PhysicalDeviceDescriptorIndexingFeatures as *mut std::ffi::c_void;
                self.instance.get_physical_device_features2(*it, &mut device_features);

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
            if builder.window.is_some() {
                let surface_khr = khr::Win32Surface::new(&self.entry, &self.instance);
                let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
                    .hinstance(win32::GetModuleHandleA(null_mut()))
                    .hwnd(builder.window.unwrap());
                    
                surface = Some(surface_khr.create_win32_surface(&create_info, None).unwrap());
            } else {
                surface = None;
            };
        }

        // Find the proper queue family indices 
        let mut graphics_family_index = None;
        let mut surface_family_index = None;
        unsafe {           
            let queue_family_properties = self.instance.get_physical_device_queue_family_properties(physical_device);
            for (index, it) in queue_family_properties.iter().enumerate() {
                if it.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_family_index = Some(index as u32);
                }

                if builder.window.is_some() {
                    let surface_khr = khr::Surface::new(&self.entry, &self.instance);
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

            logical_device = self.instance.create_device(physical_device, &create_info, None).unwrap();
            graphics_queue = Some(logical_device.get_device_queue(graphics_family_index.unwrap(), 0));

            if surface_family_index.is_some() {
                presentation_queue = Some(logical_device.get_device_queue(surface_family_index.unwrap(), 0));
            } else {
                presentation_queue = None;
            }
        }

        return Ok(Box::new(VulkanDevice{
            logical:  logical_device,
            physical: physical_device,

            graphics_queue:     graphics_queue,
            presentation_queue: presentation_queue,

            graphics_family_index: graphics_family_index,
            surface_family_index:  surface_family_index,

            surface: surface
        }));
    }
}

impl ModuleCompileTime for VulkanGPU {
    fn new() -> Result<Self, String> {
        let entry = unsafe{ Entry::new().unwrap() };

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
        let instance = unsafe{ entry.create_instance(&create_info, None).unwrap() };

        Ok(Self {
            entry:    entry,
            instance: instance,
        })
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<AssetManager>()
    }
}

impl ModuleRuntime for VulkanGPU {
    fn post_init(&mut self, _engine: &mut Engine) {
    }
}

pub struct VulkanDevice {
    logical:  ash::Device,
    physical: vk::PhysicalDevice,

    graphics_queue:     Option<vk::Queue>,
    presentation_queue: Option<vk::Queue>,

    graphics_family_index:  Option<u32>,
    surface_family_index:   Option<u32>,

    #[cfg(target_os = "windows")]
    surface: Option<vk::SurfaceKHR>,
}

impl Device for VulkanDevice {

}