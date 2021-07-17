use crate::Format;
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

mod pipeline;
pub use pipeline::*;

mod context;
pub use context::*;

pub fn vk_format(format: Format) -> vk::Format {
	match format {
		Format::Undefined => vk::Format::UNDEFINED,
		Format::RGB_U8 => vk::Format::R8G8B8_UINT,
		Format::RGB_U8_SRGB => vk::Format::R8G8B8_SRGB,
		Format::RGBA_U8 => vk::Format::R8G8B8A8_UNORM,
		Format::RGBA_U8_SRGB => vk::Format::R8G8B8A8_SRGB,
		Format::RGBA_F16 => vk::Format::R16G16B16A16_SFLOAT,
		Format::BGR_U8_SRGB => vk::Format::B8G8R8A8_SRGB,
	}
}
