use crate::FontImporter;
use crate::MeshGltfImporter;
use gpu::Gpu;

use engine::{
	Builder,
	Module,
};

use resources::Importer;

pub struct Graphics;

impl Module for Graphics {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Gpu>()
			.register(FontImporter::variant(&["ttf"]))
			.register(MeshGltfImporter::variant(&["gltf", "glb"]))
	}
}
