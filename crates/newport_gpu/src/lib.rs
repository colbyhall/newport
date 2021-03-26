//! This crate is the HAL for the GPU. Currently Vulkan is the only back end available. The design and architecture
//! was originally concepted after reading http://alextardif.com/RenderingAbstractionLayers.html
//!
//! # Warning
//! 
//! This package is still in a very early state. The API is currently super volatile. I would not 
//! recommend using this package if you don't plan on handling the unknown future changes. 
//! 
//! # Goals
//! 
//! * Abstraction layer should be as lightweight as possible. As many API layer specfic concepts should be 
//!    hidden from the user
//!
//! * Abstraction layer should be as simple as possible. There will be code complexity that is unavoidable but 
//!   they should be rare. If the user ends up spending too much time debugging just to get to the meat of 
//!   their calls then we have failed
//! 
//! * Abstraction layer should be easy to maintain and add on. The hope is that the above points aid this goal
//!
//! # Needs
//! 
//! * Ability to create multiple devices to allow multiple GPU work if desired
//! * Create, upload, and destroy resources (buffers, textures, shaders, pipelines, etc)
//! * Gather, submit, and wait on command work from various passes, in a multicore-compatible way
//! * Automatic device memory management

pub mod vk;

#[cfg(target_os = "windows")]
pub use vk::VulkanGPU as SelectedGPU;

pub trait GPU {
    fn new_device(&self, builder: DeviceBuilder) -> Result<Box<dyn Device>, DeviceCreateError>;
}

pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

pub trait Device {
    
}

use newport_os::window::WindowHandle;

pub struct DeviceBuilder {
    window: Option<WindowHandle>,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        Self {
            window: None,
        }
    }

    pub fn present_to(mut self, window: WindowHandle) -> Self {
        self.window = Some(window);
        self
    }
}