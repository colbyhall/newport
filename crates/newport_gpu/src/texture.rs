use crate::{
	api,
	serde,
};

use bitflags::bitflags;
use serde::{
	Deserialize,
	Serialize,
};

use std::sync::Arc;

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
