use crate::shader::ShaderVariant;
use crate::GraphicsPipeline;
use crate::GraphicsPipelineDescription;
use crate::Shader;
use crate::{
	api,

	Buffer,
	BufferUsage,
	Format,
	GraphicsCommandBuffer,
	GraphicsRecorder,
	MemoryType,
	Receipt,
	RenderPass,
	ResourceCreateError,
	Texture,
	TextureUsage,
};

use std::sync::Arc;

#[derive(Debug)]
pub enum DeviceCreateError {
	Unknown,
	NoValidPhysicalDevice,
}

#[derive(Clone)]
pub struct Device(pub(crate) Arc<api::Device>);

impl Device {
	pub fn create_buffer<T: Sized>(
		&self,
		usage: BufferUsage,
		memory: MemoryType,
		len: usize,
	) -> Result<Buffer<T>, ResourceCreateError> {
		let inner = api::Buffer::new(
			self.0.clone(),
			usage,
			memory,
			std::mem::size_of::<T>() * len,
		)?;
		Ok(Buffer {
			api: inner,
			phantom: Default::default(),
			len,
		})
	}

	pub fn create_texture(
		&self,
		usage: TextureUsage,
		memory: MemoryType,
		format: Format,
		width: u32,
		height: u32,
		depth: u32,
	) -> Result<Texture, ResourceCreateError> {
		let inner = api::Texture::new(self.0.clone(), memory, usage, format, width, height, depth)?;
		Ok(Texture(inner))
	}

	pub fn create_render_pass(
		&self,
		colors: Vec<Format>,
		depth: Option<Format>,
	) -> Result<RenderPass, ResourceCreateError> {
		let inner = api::RenderPass::new(self.0.clone(), colors, depth)
			.map_err(|_| ResourceCreateError::Unknown)?;
		Ok(RenderPass(inner))
	}

	pub fn create_graphics_pipeline(
		&self,
		description: GraphicsPipelineDescription,
	) -> Result<GraphicsPipeline, ResourceCreateError> {
		let inner = api::GraphicsPipeline::new(self.0.clone(), description)
			.map_err(|_| ResourceCreateError::Unknown)?;
		Ok(GraphicsPipeline(inner))
	}

	pub fn create_shader(
		&self,
		binary: &[u8],
		variant: ShaderVariant,
		main: &str,
	) -> Result<Shader, ResourceCreateError> {
		let inner = api::Shader::new(self.0.clone(), binary, variant, main.to_string())
			.map_err(|_| ResourceCreateError::Unknown)?;
		Ok(Shader(inner))
	}

	pub fn create_graphics_recorder(&self) -> GraphicsRecorder {
		let mut inner = api::GraphicsCommandBuffer::new(self.0.clone()).unwrap();
		inner.begin();
		GraphicsRecorder(inner)
	}

	pub fn acquire_backbuffer(&self) -> Texture {
		Texture(self.0.acquire_backbuffer())
	}

	pub fn submit_graphics(
		&self,
		mut command_buffers: Vec<GraphicsCommandBuffer>,
		wait_on: &[Receipt],
	) -> Receipt {
		let mut api_buffers = Vec::with_capacity(command_buffers.len());
		command_buffers
			.drain(..)
			.for_each(|x| api_buffers.push(x.0));

		self.0.submit_graphics(api_buffers, wait_on)
	}

	pub fn display(&self, wait_on: &[Receipt]) {
		self.0.display(wait_on)
	}

	pub fn wait_for_idle(&self) {
		self.0.wait_for_idle()
	}
}
