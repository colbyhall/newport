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

#[cfg(feature = "vulkan")]
pub use vk::*;

use newport_asset::AssetManager;
use newport_engine::*;
use newport_os::window::WindowHandle;

pub struct GPU {
    instance: Instance,
    device:   Option<Device>,
}

impl GPU {
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }
}

impl ModuleCompileTime for GPU {
    fn new() -> Self {
        let instance = Instance::new().unwrap();
        Self{
            instance: instance,
            device:   None,
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<AssetManager>()
    }
}

impl ModuleRuntime for GPU {
    fn post_init(&mut self, engine: &mut Engine) {
        let builder = DeviceBuilder::new()
            .present_to(engine.window().handle());
        self.device = self.instance().new_device(builder).ok();
    }
}

#[derive(Debug)]
pub enum InstanceCreateError {
    FailedToLoadLibrary,
    IncompatibleDriver,
    Unknown,
}

pub trait GenericInstance: Sized + 'static {
    fn new() -> Result<Self, InstanceCreateError>;
    
    type Device: GenericDevice;
    fn new_device(&self, builder: DeviceBuilder) -> Result<Self::Device, DeviceCreateError>;
}

#[derive(Debug)]
pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

pub trait GenericDevice {

}

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

// Type of memory allocations that buffers or textures can be allocated from
pub enum Memory_Type {
    HostVisible, // Able to be uploaded to by mapping memory. Slower to access. Faster to write to
    DeiceLocal,  // Able to be uploaded to by using commands. Faster to access. Slower to write to
}

pub enum Format {
    Undefined,
    
    RGB_U8,
    RGB_U8_SRGB,
    RGBA_U8,
    RGBA_U8_SRGB,
    
    RGBA_F16,

    BGR_U8_SRGB,    
}