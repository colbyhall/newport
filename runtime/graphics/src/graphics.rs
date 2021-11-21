use crate::FontImporter;
use crate::MeshGltfImporter;
use crate::{
	FontCollection,
	Mesh,
};
use gpu::Gpu;

use engine::{
	Builder,
	Module,
};

use resources::{
	Importer,
	Resource,
};

pub struct Graphics;

impl Module for Graphics {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Gpu>()
			.register(FontCollection::variant())
			.register(FontImporter::variant(&["ttf"]))
			.register(Mesh::variant())
			.register(MeshGltfImporter::variant(&["gltf", "glb"]))
	}
}
