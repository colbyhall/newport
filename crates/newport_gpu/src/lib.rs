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

use bitflags::*;

#[cfg(feature = "vulkan")]
mod vk;

#[cfg(feature = "vulkan")]
use vk::*;

pub mod shaders;

#[derive(Debug)]
pub enum InstanceCreateError {
    FailedToLoadLibrary,
    IncompatibleDriver,
    Unknown,
}

#[derive(Clone)]
pub struct Instance {
    inner: Arc<InstanceInner>,
}

impl Instance {
    pub fn new() -> Result<Self, InstanceCreateError> {
        let inner = InstanceInner::new()?;
        Ok(Self{ inner: inner })
    }

    pub fn create_device(&self, window: Option<WindowHandle>) -> Result<Device, DeviceCreateError> {
        let inner = DeviceInner::new(self.inner.clone(), window)?;
        Ok(Device{ inner: inner })
    }
}

#[derive(Clone)]
pub struct Receipt {
    inner: ReceiptInner,
}

impl Receipt {
    pub fn wait(self) -> bool {
        self.inner.wait()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
}

#[derive(Debug)]
pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

#[derive(Clone)]
pub struct Device {
    inner: Arc<DeviceInner>
}

impl Device {
    pub fn create_buffer(&self, usage: BufferUsage, memory: MemoryType, size: usize) -> Result<Buffer, ResourceCreateError> {
        let inner = BufferInner::new(self.inner.clone(), usage, memory, size)?;
        Ok(Buffer{ inner: inner })
    }

    pub fn create_texture(&self, usage: TextureUsage, memory: MemoryType, format: Format, width: u32, height: u32, depth: u32, wrap: Wrap, min_filter: Filter, mag_filter: Filter) -> Result<Texture, ResourceCreateError> {
        let inner = TextureInner::new(self.inner.clone(), memory, usage, format, width, height, depth, wrap, min_filter, mag_filter)?;
        Ok(Texture{ inner: inner })
    }

    pub fn create_render_pass(&self, colors: Vec<Format>, depth: Option<Format>) -> Result<RenderPass, ()> {
        let inner = RenderPassInner::new(self.inner.clone(), colors, depth)?;
        Ok(RenderPass{ inner: inner })
    }

    pub fn create_shader(&self, contents: &[u8], variant: ShaderVariant, main: String) -> Result<Shader, ()> {
        let inner = ShaderInner::new(self.inner.clone(), contents, variant, main)?;
        Ok(Shader{ inner: inner })
    }

    pub fn create_pipeline(&self, desc: PipelineDescription) -> Result<Pipeline, ()> {
        let inner = PipelineInner::new(self.inner.clone(), desc)?;
        Ok(Pipeline{ inner: inner })
    }

    pub fn create_graphics_context(&self) -> Result<GraphicsContext, ()> {
        let inner = GraphicsContextInner::new(self.inner.clone())?;
        Ok(GraphicsContext{ inner: inner })
    }

    pub fn acquire_backbuffer(&self) -> Texture {
        Texture{ inner: self.inner.acquire_backbuffer() }
    }

    pub fn submit_graphics(&self, contexts: Vec<GraphicsContext>, wait_on: &[Receipt]) -> Receipt {
        self.inner.submit_graphics(contexts, wait_on)
    }

    pub fn display(&self, wait_on: &[Receipt]) {
        self.inner.display(wait_on)
    }

    pub fn update_bindless(&self) {
        self.inner.update_bindless()
    }

    pub fn wait_for_idle(&self) {
        self.inner.wait_for_idle()
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
pub struct Buffer {
    inner: Arc<BufferInner>,
}

impl Buffer {
    pub fn copy_to<T>(&self, data: &[T]) {
        self.inner.copy_to::<T>(data)
    }

    pub fn bindless(&self) -> Option<u32> {
        self.inner.bindless()
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
pub struct Texture {
    inner: Arc<TextureInner>,
}

impl Texture {
    pub fn format(&self) -> Format { 
        self.inner.format()
    }

    pub fn width(&self) -> u32 { 
        self.inner.width()
    }

    pub fn height(&self) -> u32 { 
        self.inner.height()
    }

    pub fn depth(&self) -> u32 { 
        self.inner.depth()
    }

    pub fn bindless(&self) -> Option<u32> {
        self.inner.bindless()
    }
}

#[derive(Clone)]
pub struct RenderPass {
    inner: Arc<RenderPassInner>,
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShaderVariant {
    Vertex,
    Pixel,
}

pub struct Shader {
    inner: Arc<ShaderInner>,
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

pub trait Vertex {
    fn attributes() -> Vec<VertexAttribute>;
}

pub struct PipelineBuilder {
    desc: PipelineDescription,
}

impl PipelineBuilder {
    pub fn new_graphics(render_pass: RenderPass) -> Self {
        let desc = GraphicsPipelineDescription{
            render_pass: render_pass,
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
pub struct Pipeline {
    inner: Arc<PipelineInner>,
}

pub struct GraphicsContext {
    inner: GraphicsContextInner,
}

impl GraphicsContext {
    pub fn begin(&mut self) {
        self.inner.begin();
    }

    pub fn end(&mut self) {
        self.inner.end();
    }

    pub fn resource_barrier_texture(&mut self, texture: &Texture, old_layout: Layout, new_layout: Layout) {
        self.inner.resource_barrier_texture(texture.inner.clone(), old_layout, new_layout);
    }

    pub fn copy_buffer_to_texture(&mut self, dst: &Texture, src: &Buffer) {
        self.inner.copy_buffer_to_texture(dst.inner.clone(), src.inner.clone());
    }

    pub fn begin_render_pass(&mut self, render_pass: &RenderPass, attachments: &[&Texture]) {
        let mut a = Vec::with_capacity(attachments.len());
        attachments.iter().for_each(|e| a.push(e.inner.clone()) );

        self.inner.begin_render_pass(render_pass.inner.clone(), &a[..]);
    }

    pub fn end_render_pass(&mut self) {
        self.inner.end_render_pass();
    }

    pub fn clear(&mut self, color: Color) -> &mut Self {
        self.inner.clear(color);
        self
    }

    pub fn bind_pipeline(&mut self, pipeline: &Pipeline) {
        self.inner.bind_pipeline(pipeline.inner.clone());
    }

    pub fn bind_scissor(&mut self, scissor: Option<Rect>) {
        self.inner.bind_scissor(scissor);
    }

    pub fn bind_vertex_buffer(&mut self, buffer: &Buffer){
        self.inner.bind_vertex_buffer(buffer.inner.clone());
    }

    pub fn draw(&mut self, vertex_count: usize, first_vertex: usize) {
        self.inner.draw(vertex_count, first_vertex);
    }

    pub fn push_constants<T>(&mut self, t: T) {
        self.inner.push_constants::<T>(t);
    }
}