use crate::Device;
use crate::{
	api,

	BufferUsage,
	Gpu,

	GraphicsRecorder,
	MemoryType,
	Result,
};

use bitflags::bitflags;

use asset::Asset;
use asset::Importer;
use serde::{
	self,
	Deserialize,
	Serialize,
};

use std::sync::Arc;

use image::LoadResult;
use stb_image::image;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Format {
	Undefined,

	RGB_U8,
	RGB_U8_SRGB,
	RGBA_U8,
	RGBA_U8_SRGB,

	RGBA_F16,

	RGBA_F32,

	BGR_U8_SRGB,

	D24_S8,
}

impl Format {
	pub fn is_depth(self) -> bool {
		self == Format::D24_S8
	}

	pub fn is_stencil(self) -> bool {
		self == Format::D24_S8
	}

	pub fn is_color(self) -> bool {
		!self.is_depth()
	}
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Wrap {
	Clamp,
	Repeat,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Filter {
	Nearest,
	Linear,
}

pub struct TextureBuilder<'a> {
	usage: TextureUsage,
	memory: MemoryType,
	format: Format,
	width: u32,
	height: u32,
	depth: u32,

	device: Option<&'a Device>,
}

impl<'a> TextureBuilder<'a> {
	pub fn memory(mut self, memory: MemoryType) -> Self {
		self.memory = memory;
		self
	}

	pub fn device(mut self, device: &'a Device) -> Self {
		self.device = Some(device);
		self
	}

	pub fn spawn(self) -> Result<Texture> {
		let device = match self.device {
			Some(device) => device,
			None => Gpu::device(),
		};

		Ok(Texture(api::Texture::new(
			device.0.clone(),
			self.memory,
			self.usage,
			self.format,
			self.width,
			self.height,
			self.depth,
		)?))
	}
}

#[derive(Clone)]
pub struct Texture(pub(crate) Arc<api::Texture>);

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

	pub fn builder<'a>(
		usage: TextureUsage,
		format: Format,
		width: u32,
		height: u32,
		depth: u32,
	) -> TextureBuilder<'a> {
		TextureBuilder {
			usage,
			memory: MemoryType::DeviceLocal,
			format,
			width,
			height,
			depth,

			device: None,
		}
	}

	pub fn new(
		usage: TextureUsage,
		format: Format,
		width: u32,
		height: u32,
		depth: u32,
	) -> Result<Self> {
		Texture::builder(usage, format, width, height, depth).spawn()
	}

	pub fn new_in(
		usage: TextureUsage,
		format: Format,
		width: u32,
		height: u32,
		depth: u32,
		device: &Device,
	) -> Result<Self> {
		Texture::builder(usage, format, width, height, depth)
			.device(device)
			.spawn()
	}
}

impl Asset for Texture {}

#[derive(Serialize, Deserialize)]
pub(crate) struct TextureImporter {
	#[serde(default)]
	srgb: bool,
}

impl Importer for TextureImporter {
	type Target = Texture;

	fn import(&self, bytes: &[u8]) -> asset::Result<Self::Target> {
		Ok(match image::load_from_memory(bytes) {
			LoadResult::Error(err) => panic!("Failed to load texture from file due to {}", err),
			LoadResult::ImageU8(image) => {
				let pixels = if image.depth == 3 {
					let cap = (image.data.len() / 3) * 4;
					let mut result = Vec::with_capacity(cap);

					for index in 0..image.data.len() / 3 {
						let base = index * 3;
						let r = image.data[base];
						let g = image.data[base + 1];
						let b = image.data[base + 2];
						let a = 255;

						result.push(r);
						result.push(g);
						result.push(b);
						result.push(a);
					}

					result
				} else {
					assert_eq!(
						image.depth, 4,
						"Currently vulkan only supports 4 byte formats"
					);
					image.data.clone()
				};

				let pixel_buffer = crate::Buffer::new(
					BufferUsage::TRANSFER_SRC,
					MemoryType::HostVisible,
					pixels.len(),
				)?;
				pixel_buffer.copy_to(&pixels[..]);

				let format = if self.srgb {
					Format::RGBA_U8_SRGB
				} else {
					Format::RGBA_U8
				};

				let gpu_texture = Texture::builder(
					TextureUsage::TRANSFER_DST | TextureUsage::SAMPLED,
					format,
					image.width as u32,
					image.height as u32,
					1,
				)
				.spawn()?;

				GraphicsRecorder::new()
					.resource_barrier_texture(&gpu_texture, Layout::Undefined, Layout::TransferDst)
					.copy_buffer_to_texture(&gpu_texture, &pixel_buffer)
					.resource_barrier_texture(
						&gpu_texture,
						Layout::TransferDst,
						Layout::ShaderReadOnly,
					)
					.submit()
					.wait();

				gpu_texture
			}
			_ => unimplemented!(),
		})
	}
}
