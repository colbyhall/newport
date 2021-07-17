use super::Device;
use crate::ShaderVariant;

use ash::version::DeviceV1_0;
use ash::vk;

use std::slice::from_raw_parts;
use std::sync::Arc;

pub struct Shader {
	pub owner: Arc<Device>,

	pub variant: ShaderVariant,
	pub module: vk::ShaderModule,
	pub main: String,
}

impl Shader {
	pub fn new(
		owner: Arc<Device>,
		contents: &[u8],
		variant: ShaderVariant,
		main: String,
	) -> Result<Arc<Shader>, ()> {
		let contents =
			unsafe { from_raw_parts(contents.as_ptr() as *const u32, contents.len() / 4) };

		let create_info = vk::ShaderModuleCreateInfo::builder().code(contents);

		let shader = unsafe { owner.logical.create_shader_module(&create_info, None) };
		if shader.is_err() {
			return Err(());
		}
		let shader = shader.unwrap();

		Ok(Arc::new(Shader {
			owner: owner,

			variant: variant,
			module: shader,
			main: main,
		}))
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		// todo!();
	}
}
