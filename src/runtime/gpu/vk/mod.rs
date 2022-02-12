use crate::{
	CompareOp,
	Format,
};
use ash::vk;

pub const ENABLED_LAYER_NAMES: [*const i8; 1] =
	[b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8];

mod instance;
pub use instance::*;

mod device;
pub use device::*;

mod receipt;
pub use receipt::*;

mod buffer;
pub use buffer::*;

mod texture;
pub use texture::*;

mod render_pass;
pub use render_pass::*;

mod shader;
pub use shader::*;

mod graphics_pipeline;
pub use graphics_pipeline::*;

mod command_buffer;
pub use command_buffer::*;

mod sampler;
pub use sampler::*;

pub fn vk_format(format: Format) -> vk::Format {
	match format {
		Format::Undefined => vk::Format::UNDEFINED,
		Format::RGB_U8 => vk::Format::R8G8B8_UINT,
		Format::RGB_U8_SRGB => vk::Format::R8G8B8_SRGB,
		Format::RGBA_U8 => vk::Format::R8G8B8A8_UNORM,
		Format::RGBA_U8_SRGB => vk::Format::R8G8B8A8_SRGB,
		Format::RGBA_F16 => vk::Format::R16G16B16A16_SFLOAT,
		Format::RGBA_F32 => vk::Format::R32G32B32A32_SFLOAT,
		Format::BGR_U8_SRGB => vk::Format::B8G8R8A8_SRGB,
		Format::Depth24_Stencil8 => vk::Format::D24_UNORM_S8_UINT,
		Format::Depth16 => vk::Format::D16_UNORM,
	}
}

pub fn vk_format_aspect_mask(format: Format) -> vk::ImageAspectFlags {
	if format.is_color() {
		return vk::ImageAspectFlags::COLOR;
	}

	let mut result = vk::ImageAspectFlags::empty();
	if format.is_depth() {
		result |= vk::ImageAspectFlags::DEPTH;
	}
	if format.is_stencil() {
		result |= vk::ImageAspectFlags::STENCIL;
	}

	result
}

pub fn vk_compare_op(compare_op: CompareOp) -> vk::CompareOp {
	match compare_op {
		CompareOp::Never => vk::CompareOp::NEVER,
		CompareOp::Less => vk::CompareOp::LESS,
		CompareOp::Equal => vk::CompareOp::EQUAL,
		CompareOp::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
		CompareOp::Greater => vk::CompareOp::GREATER,
		CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
		CompareOp::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
		CompareOp::Always => vk::CompareOp::ALWAYS,
	}
}

pub use ash::vk::Result as Error;
