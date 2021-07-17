use crate::{
	asset,

	engine,
	gpu::Gpu,
	FontCollection,
	Mesh,
	Texture,
};

use engine::{
	Builder,
	Module,
};

use asset::AssetVariant;

pub struct Graphics;

impl Module for Graphics {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Gpu>()
			.register(AssetVariant::new::<Texture>(&["texture", "tex"]))
			.register(AssetVariant::new::<FontCollection>(&["font"]))
			.register(AssetVariant::new::<Mesh>(&["mesh"]))
	}
}
