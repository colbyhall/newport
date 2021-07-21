use crate::{
	api,
	asset,
	engine,
	serde,

	BufferUsage,
	Gpu,

	MemoryType,
};

use engine::Engine;

use bitflags::bitflags;

use asset::Asset;
use asset::Importer;
use serde::{
	Deserialize,
	Serialize,
};

use std::sync::Arc;

use image::LoadResult;
use stb_image::image;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum Wrap {
	Clamp,
	Repeat,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum Filter {
	Nearest,
	Linear,
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
}

impl Asset for Texture {}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
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
				let engine = Engine::as_ref();
				let gpu = engine.module::<Gpu>().unwrap();
				let device = gpu.device();

				assert_eq!(
					image.depth, 4,
					"Currently vulkan only supports 4 byte formats"
				);

				let pixel_buffer = device
					.create_buffer(
						BufferUsage::TRANSFER_SRC,
						MemoryType::HostVisible,
						image.data.len(),
					)
					.unwrap();
				pixel_buffer.copy_to(&image.data[..]);

				let format = if self.srgb {
					Format::RGBA_U8_SRGB
				} else {
					Format::RGBA_U8
				};

				let gpu_texture = device
					.create_texture(
						TextureUsage::TRANSFER_DST | TextureUsage::SAMPLED,
						MemoryType::DeviceLocal,
						format,
						image.width as u32,
						image.height as u32,
						1,
					)
					.unwrap();

				let gfx = device
					.create_graphics_recorder()
					.resource_barrier_texture(&gpu_texture, Layout::Undefined, Layout::TransferDst)
					.copy_buffer_to_texture(&gpu_texture, &pixel_buffer)
					.resource_barrier_texture(
						&gpu_texture,
						Layout::TransferDst,
						Layout::ShaderReadOnly,
					)
					.finish();

				let receipt = device.submit_graphics(vec![gfx], &[]);
				receipt.wait();

				gpu_texture
			}
			_ => unimplemented!(),
		})
	}
}
