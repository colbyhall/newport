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

use newport_os::window::WindowHandle;
use newport_math::{ Rect, Color };

use std::mem::size_of;
use std::sync::{ Arc };
use std::convert::Into;

use bitflags::*;

#[cfg(feature = "vulkan")]
mod vk;

#[cfg(feature = "vulkan")]
use vk as api;

pub mod shaders;

#[derive(Debug)]
pub enum InstanceCreateError {
    FailedToLoadLibrary,
    IncompatibleDriver,
    Unknown,
}

#[derive(Clone)]
pub struct Instance(Arc<api::Instance>);

impl Instance {
    pub fn new() -> Result<Self, InstanceCreateError> {
        let inner = api::Instance::new()?;
        Ok(Self(inner))
    }

    pub fn create_device(&self, window: Option<WindowHandle>) -> Result<Device, DeviceCreateError> {
        let inner = api::Device::new(self.0.clone(), window)?;
        Ok(Device(inner))
    }
}

pub use api::Receipt as Receipt;

#[derive(Debug)]
pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

#[derive(Clone)]
pub struct Device(Arc<api::Device>);

impl Device {
    pub fn create_buffer(&self, usage: BufferUsage, memory: MemoryType, size: usize) -> Result<Buffer, ResourceCreateError> {
        let inner = api::Buffer::new(self.0.clone(), usage, memory, size)?;
        Ok(Buffer(inner))
    }

    pub fn create_texture(&self, usage: TextureUsage, memory: MemoryType, format: Format, width: u32, height: u32, depth: u32, wrap: Wrap, min_filter: Filter, mag_filter: Filter) -> Result<Texture, ResourceCreateError> {
        let inner = api::Texture::new(self.0.clone(), memory, usage, format, width, height, depth, wrap, min_filter, mag_filter)?;
        Ok(Texture(inner))
    }

    pub fn create_render_pass(&self, colors: Vec<Format>, depth: Option<Format>) -> Result<RenderPass, ()> {
        let inner = api::RenderPass::new(self.0.clone(), colors, depth)?;
        Ok(RenderPass(inner))
    }

    pub fn create_shader(&self, contents: &[u8], variant: ShaderVariant, main: String) -> Result<Shader, ()> {
        let inner = api::Shader::new(self.0.clone(), contents, variant, main)?;
        Ok(Shader(inner))
    }

    pub fn create_pipeline(&self, desc: PipelineDescription) -> Result<Pipeline, ()> {
        let inner = api::Pipeline::new(self.0.clone(), desc)?;
        Ok(Pipeline(inner))
    }

    pub fn create_graphics_context(&self) -> Result<GraphicsContext, ()> {
        let inner = api::GraphicsContext::new(self.0.clone())?;
        Ok(GraphicsContext(inner))
    }

    pub fn acquire_backbuffer(&self) -> Texture {
        Texture(self.0.acquire_backbuffer())
    }

    pub fn submit_graphics(&self, mut contexts: Vec<GraphicsContext>, wait_on: &[Receipt]) -> Receipt {
        let mut api_contexts = Vec::with_capacity(contexts.len());
        contexts.drain(..).for_each(|x| api_contexts.push(x.0));

        self.0.submit_graphics(api_contexts, wait_on)
    }

    pub fn display(&self, wait_on: &[Receipt]) {
        self.0.display(wait_on)
    }

    pub fn update_bindless(&self) {
        self.0.update_bindless()
    }

    pub fn wait_for_idle(&self) {
        self.0.wait_for_idle()
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

#[derive(Clone)]
pub struct Buffer(Arc<api::Buffer>);

impl Buffer {
    pub fn copy_to<T>(&self, data: &[T]) {
        self.0.copy_to::<T>(data)
    }

    pub fn bindless(&self) -> Option<u32> {
        self.0.bindless()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ResourceCreateError {
    Unknown,
    OutOfMemory,
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

#[derive(Clone)]
pub struct Texture(Arc<api::Texture>);

impl Texture {
    pub fn format(&self) -> Format { 
        self.0.format()
    }

    pub fn width(&self) -> u32 { 
        self.0.width()
    }

    pub fn height(&self) -> u32 { 
        self.0.height()
    }

    pub fn depth(&self) -> u32 { 
        self.0.depth()
    }

    pub fn bindless(&self) -> Option<u32> {
        self.0.bindless()
    }
}

#[derive(Clone)]
pub struct RenderPass(Arc<api::RenderPass>);


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShaderVariant {
    Vertex,
    Pixel,
}

pub struct Shader(Arc<api::Shader>);

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
    Uint32,
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
            VertexAttribute::Uint32  => 4,
            VertexAttribute::Float32 => 4,
            VertexAttribute::Vector2 => 8,
            VertexAttribute::Vector3 => 12,
            VertexAttribute::Vector4 => 16,
            VertexAttribute::Color   => 16,
        }
    }
}

pub trait Vertex {
    fn attributes() -> Vec<VertexAttribute>;
}

pub struct PipelineBuilder {
    desc: PipelineDescription,
}

impl PipelineBuilder {
    pub fn new_graphics(render_pass: &RenderPass) -> Self {
        let desc = GraphicsPipelineDescription{
            render_pass: render_pass.clone(),
            shaders:     Vec::new(),

            vertex_attributes: Vec::new(),

            draw_mode:  DrawMode::Fill,
            line_width: 1.0,

            cull_mode:  CullMode::empty(),
            color_mask: ColorMask::all(),

            blend_enabled: false,

            src_color_blend_factor: BlendFactor::One,
            dst_color_blend_factor: BlendFactor::One,
            color_blend_op:         BlendOp::Add,

            src_alpha_blend_factor: BlendFactor::One,
            dst_alpha_blend_factor: BlendFactor::One,
            alpha_blend_op:         BlendOp::Add,

            depth_test:    true, 
            depth_write:   true,
            depth_compare: CompareOp::Less,

            push_constant_size: 0,
        };
        Self { desc: PipelineDescription::Graphics(desc) }
    }

    pub fn shaders(mut self, shaders: Vec<Shader>) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.shaders = shaders,
            _ => unreachable!()
        }
        self
    }

    pub fn vertex<T: Vertex>(mut self) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.vertex_attributes = T::attributes(),
            _ => unreachable!()
        }
        self
    }

    pub fn draw_mode(mut self, mode: DrawMode) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.draw_mode = mode,
            _ => unreachable!()
        }
        self
    }

    pub fn line_width(mut self, width: f32) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.line_width = width,
            _ => unreachable!()
        }
        self
    }
    
    pub fn push_constant_size<T: Sized>(mut self) -> Self {
        assert!(size_of::<T>() <= 128);
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.push_constant_size = size_of::<T>(),
            _ => unreachable!()
        }
        self
    }

    pub fn enable_blend(mut self) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.blend_enabled = true,
            _ => unreachable!()
        }
        self
    }

    pub fn src_alpha_blend(mut self, factor: BlendFactor) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.src_alpha_blend_factor = factor,
            _ => unreachable!()
        }
        self
    }

    pub fn build(self) -> PipelineDescription {
        self.desc
    }
}

pub struct GraphicsPipelineDescription {
    pub render_pass:  RenderPass,
    pub shaders:      Vec<Shader>,
    
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
    Graphics(GraphicsPipelineDescription),
    Compute,
}

#[derive(Clone)]
pub struct Pipeline(Arc<api::Pipeline>);

pub struct GraphicsContext(api::GraphicsContext);

impl GraphicsContext {
    pub fn begin(&mut self) {
        self.0.begin();
    }

    pub fn end(&mut self) {
        self.0.end();
    }

    pub fn resource_barrier_texture(&mut self, texture: &Texture, old_layout: Layout, new_layout: Layout) {
        self.0.resource_barrier_texture(texture.0.clone(), old_layout, new_layout);
    }

    pub fn copy_buffer_to_texture(&mut self, dst: &Texture, src: &Buffer) {
        self.0.copy_buffer_to_texture(dst.0.clone(), src.0.clone());
    }

    pub fn begin_render_pass(&mut self, render_pass: &RenderPass, attachments: &[&Texture]) {
        let mut a = Vec::with_capacity(attachments.len());
        attachments.iter().for_each(|e| a.push(e.0.clone()) );

        self.0.begin_render_pass(render_pass.0.clone(), &a[..]);
    }

    pub fn end_render_pass(&mut self) {
        self.0.end_render_pass();
    }

    pub fn clear(&mut self, color: impl Into<Color>) -> &mut Self {
        self.0.clear(color.into());
        self
    }

    pub fn bind_pipeline(&mut self, pipeline: &Pipeline) {
        self.0.bind_pipeline(pipeline.0.clone());
    }

    pub fn bind_scissor(&mut self, scissor: Option<Rect>) {
        self.0.bind_scissor(scissor);
    }

    pub fn bind_vertex_buffer(&mut self, buffer: &Buffer){
        self.0.bind_vertex_buffer(buffer.0.clone());
    }

    pub fn bind_index_buffer(&mut self, buffer: &Buffer) {
        self.0.bind_index_buffer(buffer.0.clone());
    }

    pub fn draw(&mut self, vertex_count: usize, first_vertex: usize) {
        self.0.draw(vertex_count, first_vertex);
    }

    pub fn draw_indexed(&mut self, index_count: usize, first_index: usize) {
        self.0.draw_indexed(index_count, first_index);
    }

    pub fn push_constants<T>(&mut self, t: T) {
        self.0.push_constants::<T>(t);
    }
}