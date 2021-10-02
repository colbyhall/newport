use crate::GraphicsPipelineImporter;
use crate::TextureImporter;
use crate::{
	Device,
	Instance,
};

use asset::Variant;
use engine::{
	Builder,
	Engine,
	Module,
};

pub struct Gpu {
	device: Device,
}

impl Gpu {
	pub fn device() -> &'static Device {
		let gpu: &Gpu = Engine::module()
			.expect("Engine must depend on Gpu module if the global device is to be used. ");
		&gpu.device
	}
}

impl Module for Gpu {
	fn new() -> Self {
		let instance = Instance::new().unwrap();
		let device = instance.create_device(Engine::window()).unwrap();

		Self { device }
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.register(Variant::new::<GraphicsPipelineImporter>(&[
				"graphics_pipeline",
			]))
			.register(Variant::new::<TextureImporter>(&["png", "psd", "jpg"]))
	}
}
