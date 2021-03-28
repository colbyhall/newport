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
use newport_core::math::{ Rect, Color };

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

pub trait GenericReceipt { }

#[derive(Debug)]
pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

pub trait GenericDevice {
    fn new(instance: Arc<Instance>, window: Option<WindowHandle>) -> Result<Arc<Self>, DeviceCreateError>;

    fn acquire_backbuffer(&self) -> (Arc<Texture>, Receipt);

    fn submit_graphics(&self, contexts: &[&GraphicsContext], wait_on: &[&Receipt]) -> Receipt;
    fn display(&self, wait_on: &[&Receipt]);
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

#[derive(Copy, Clone, Debug)]
pub enum ShaderVariant {
    Vertex,
    Pixel,
}

pub trait GenericShader {
    fn new(owner: Arc<Device>, contents: Vec<u8>, variant: ShaderVariant) -> Result<Arc<Shader>, ()>;
}

#[derive(Copy, Clone, Debug)]
pub enum DrawMode {
    Fill,
    Line,
    Point,
}

bitflags!{
    pub struct CullMode: u32 {
        const FRONT = 0b01;
        const BACK  = 0b10;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CompareOp {
    Never,
    Less,             // A < B
    Equal,            // A == B
    LessOrEqual,      // A < B || A == B
    Greater,          // A > B
    NotEqual,         // A != B
    GreaterOrEqual,   // A > B || A == B
    Always,
}

#[derive(Copy, Clone, Debug)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Copy, Clone, Debug)]
pub enum BlendFactor {
    Zero,
    One,

    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,

    SrcAlpha,
    OneMinusSrcAlpha,
}

bitflags! {
    pub struct ColorMask: u32 {
        const RED   = 0b0001;
        const GREEN = 0b0010;
        const BLUE  = 0b0100;
        const ALPHA = 0b1000;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum VertexAttribute {
    Int32,
    Float32,
    Vector2,
    Vector3,
    Vector4,
    Color,
}

impl VertexAttribute {
    pub fn size(self) -> usize {
        match self {
            VertexAttribute::Int32   => 4,
            VertexAttribute::Float32 => 4,
            VertexAttribute::Vector2 => 8,
            VertexAttribute::Vector3 => 12,
            VertexAttribute::Vector4 => 16,
            VertexAttribute::Color   => 16,
        }
    }
}

pub struct GraphicsPipelineDescription {
    pub render_pass:  Arc<RenderPass>,
    pub shaders:      Vec<Arc<Shader>>,
    
    pub vertex_attributes: Vec<VertexAttribute>,

    pub draw_mode:  DrawMode,
    pub line_width: f32,

    pub cull_mode:   CullMode,
    pub color_mask:  ColorMask,

    pub blend_enabled: bool,

    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op:         BlendOp,

    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op:         BlendOp,    

    pub depth_test:    bool,
    pub depth_write:   bool,
    pub depth_compare: CompareOp,

    /// Capped at 128 bytes
    pub push_constant_size : usize, 
}

pub enum PipelineDescription {
    Graphics(GraphicsPipelineDescription)
}

pub trait GenericPipeline {
    fn new(owner: Arc<Device>, desc: PipelineDescription) -> Result<Arc<Pipeline>, ()>;
}

pub trait GenericContext {
    fn begin(&mut self);
    fn end(&mut self);

    fn copy_buffer_to_texture(&mut self, dst: Arc<Texture>, src: Arc<Buffer>);
    fn resource_barrier_texture(&mut self, texture: Arc<Texture>, old_layout: Layout, new_layout: Layout);
}

pub trait GenericGraphicsContext<'a>: GenericContext {
    fn new(owner: Arc<Device>) -> Result<GraphicsContext<'a>, ()>;

    fn begin_render_pass(&mut self, render_pass: &'a Arc<RenderPass>, attachments: &[Arc<Texture>]);
    fn end_render_pass(&mut self);

    fn bind_scissor(&mut self, scissor: Option<Rect>);
    fn bind_pipeline(&mut self, pipeline: Arc<Pipeline>);
    fn bind_vertex_buffer(&mut self, buffer: Arc<Buffer>);

    fn draw(&mut self, vertex_count: usize, first_vertex: usize);
    fn clear(&mut self, color: Color, attachments: &[Arc<Texture>]);

    fn push_constants<T>(&mut self, t: T);
}