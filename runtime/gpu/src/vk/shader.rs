use super::Device;
use crate::ShaderVariant;

use ash::version::DeviceV1_0;
use ash::vk;

use std::slice::from_raw_parts;
use std::sync::Arc;

use crate::Result;

pub struct Shader {
	pub owner: Arc<Device>,

	pub variant: ShaderVariant,
	pub module: vk::ShaderModule,
	pub main: String,
}

impl Shader {
	pub fn new(
		owner: Arc<Device>,
		binary: &[u8],
		variant: ShaderVariant,
		main: String,
	) -> Result<Arc<Shader>> {
		let contents = unsafe { from_raw_parts(binary.as_ptr() as *const u32, binary.len() / 4) };

		let create_info = vk::ShaderModuleCreateInfo::builder().code(contents);

		let shader = unsafe { owner.logical.create_shader_module(&create_info, None)? };

		Ok(Arc::new(Shader {
			owner,

			variant,
			module: shader,
			main,
		}))
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		// todo!();
	}
}
