use crate::FontImporter;
use crate::MeshImporter;
use gpu::Gpu;

use engine::{
	Builder,
	Module,
};

pub struct Graphics;

impl Module for Graphics {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Gpu>()
			.register(asset::Variant::new::<FontImporter>(&["ttf"]))
			.register(asset::Variant::new::<MeshImporter>(&["gltf"]))
	}
}
