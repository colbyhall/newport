use crate::GraphicsPipelineImporter;
use crate::TextureImporter;
use crate::{
	Device,
	Format,
	Instance,
	RenderPass,
};

use asset::Variant;
use engine::{
	Builder,
	Engine,
	Module,
};

pub struct Gpu {
	device: Device,
	backbuffer_render_pass: RenderPass,
}

impl Gpu {
	pub fn device(&self) -> &Device {
		&self.device
	}

	pub fn backbuffer_render_pass(&self) -> &RenderPass {
		&self.backbuffer_render_pass
	}
}

impl Module for Gpu {
	fn new() -> Self {
		let engine = Engine::as_ref();

		let instance = Instance::new().unwrap();
		let device = instance.create_device(engine.window()).unwrap();

		let backbuffer_render_pass = device
			.create_render_pass(vec![Format::BGR_U8_SRGB], None)
			.unwrap();

		Self {
			device,
			backbuffer_render_pass,
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.register(Variant::new::<GraphicsPipelineImporter>(&[
				"graphics_pipeline",
			]))
			.register(Variant::new::<TextureImporter>(&["png", "psd", "jpg"]))
	}
}
