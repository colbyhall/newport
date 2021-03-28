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

use newport_os::window::WindowHandle;
use bitflags::*;

pub use std::sync::Arc;

#[derive(Debug)]
pub enum InstanceCreateError {
    FailedToLoadLibrary,
    IncompatibleDriver,
    Unknown,
}

pub trait GenericInstance: Sized + 'static {
    fn new() -> Result<Arc<Self>, InstanceCreateError>;
}

#[derive(Debug)]
pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

pub trait GenericDevice {
    fn new(instance: Arc<Instance>, window: Option<WindowHandle>) -> Result<Arc<Self>, DeviceCreateError>;
}

pub struct DeviceBuilder {
    instance: Arc<Instance>,
    window:   Option<WindowHandle>,
}

impl DeviceBuilder {
    pub fn new(instance: Arc<Instance>) -> Self {
        Self {
            instance: instance,
            window:   None,
        }
    }

    pub fn present_to(mut self, window: WindowHandle) -> Self {
        self.window = Some(window);
        self
    }

    pub fn spawn(self) -> Result<Arc<Device>, DeviceCreateError> {
        Device::new(self.instance, self.window)
    }
}

/// Type of memory allocations that buffers or textures can be allocated from
#[derive(Copy, Clone, Debug)]
pub enum MemoryType {
    /// Able to be uploaded to by mapping memory. Slower to access. Faster to write to
    HostVisible, 
    /// Able to be uploaded to by using commands. Faster to access. Slower to write to
    DeviceLocal,  
}

bitflags! {
    pub struct BufferUsage: u32 {
        const TRANSFER_SRC      = 0b000001;
        const TRANSFER_DST      = 0b000010;
        const VERTEX            = 0b000100;
        const INDEX             = 0b001000;
        const CONSTANTS         = 0b010000;
    }
}

pub enum ResourceCreateError {
    Unknown,
    OutOfMemory,
}

pub trait GenericBuffer {
    fn new(device: Arc<Device>, usage: BufferUsage, memory: MemoryType, size: usize) -> Result<Arc<Buffer>, ResourceCreateError>;
    fn copy_to<T>(&self, data: Vec<T>);
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Format {
    Undefined,
    
    RGB_U8,
    RGB_U8_SRGB,
    RGBA_U8,
    RGBA_U8_SRGB,
    
    RGBA_F16,

    BGR_U8_SRGB,    
}

bitflags! {
    pub struct TextureUsage: u32 {
        const TRANSFER_SRC      = 0b000001;
        const TRANSFER_DST      = 0b000010;
        const SAMPLED           = 0b000100;
        const COLOR_ATTACHMENT  = 0b001000;
        const DEPTH_ATTACHMENT  = 0b010000;
        const SWAPCHAIN         = 0b100000;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Layout {
    Undefined,
    General,
    ColorAttachment,
    DepthAttachment,
    TransferSrc,
    TransferDst,
    ShaderReadOnly,
    Present,
}

#[derive(Copy, Clone, Debug)]
pub enum Wrap {
    Clamp,
    Repeat,
}

#[derive(Copy, Clone, Debug)]
pub enum Filter {
    Nearest,
    Linear,
}

pub trait GenericTexture {
    fn owner(&self) -> &Arc<Device>;
    fn memory_type(&self) -> MemoryType;
    fn usage(&self) -> TextureUsage;
    fn format(&self) -> Format;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depth(&self) -> u32;
}

pub trait GenericRenderPass {
    fn new(owner: Arc<Device>, colors: Vec<Format>, depth: Option<Format>) -> Result<Arc<RenderPass>, ()>;
}