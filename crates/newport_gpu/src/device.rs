use crate::{
    api,

    BufferUsage,
    MemoryType,
    Buffer,
    Texture,
    TextureUsage,
    Format,
    ResourceCreateError,
    RenderPass,
    GraphicsContext,
    Receipt,
    Wrap,
    Filter,
};

use std::{
    sync::Arc,
};

#[derive(Debug)]
pub enum DeviceCreateError {
    Unknown,
    NoValidPhysicalDevice,
}

#[derive(Clone)]
pub struct Device(pub(crate) Arc<api::Device>);

impl Device {
    pub fn create_buffer<T: Sized>(&self, usage: BufferUsage, memory: MemoryType, len: usize) -> Result<Buffer<T>, ResourceCreateError> {
        let inner = api::Buffer::new(self.0.clone(), usage, memory, std::mem::size_of::<T>() * len)?;
        Ok(Buffer{ api: inner, phantom: Default::default(), len })
    }

    pub fn create_texture(&self, usage: TextureUsage, memory: MemoryType, format: Format, width: u32, height: u32, depth: u32, wrap: Wrap, min_filter: Filter, mag_filter: Filter) -> Result<Texture, ResourceCreateError> {
        let inner = api::Texture::new(self.0.clone(), memory, usage, format, width, height, depth, wrap, min_filter, mag_filter)?;
        Ok(Texture(inner))
    }

    pub fn create_render_pass(&self, colors: Vec<Format>, depth: Option<Format>) -> Result<RenderPass, ()> {
        let inner = api::RenderPass::new(self.0.clone(), colors, depth)?;
        Ok(RenderPass(inner))
    }

    pub fn create_graphics_context(&self) -> Result<GraphicsContext, ()> {
        let inner = api::GraphicsContext::new(self.0.clone())?;
        Ok(GraphicsContext{ api: inner })
    }

    pub fn acquire_backbuffer(&self) -> Texture {
        Texture(self.0.acquire_backbuffer())
    }

    pub fn submit_graphics(&self, mut contexts: Vec<GraphicsContext>, wait_on: &[Receipt]) -> Receipt {
        let mut api_contexts = Vec::with_capacity(contexts.len());
        contexts.drain(..).for_each(|x| api_contexts.push(x.api));

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
