#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(const_type_name)]
#![feature(trait_alias)]
#![feature(wrapping_int_impl)]

mod component;
mod entity;
mod query;
mod scene;
mod system;
mod world;

pub use {
	component::*,
	entity::*,
	query::*,
	scene::*,
	system::*,
	world::*,
};

use {
	engine::{
		Builder,
		Module,
	},
	resources::{
		Importer,
		ResourceManager,
	},
};

pub struct Ecs;
impl Module for Ecs {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<ResourceManager>()
			.register(SceneImporter::variant(&["scene"]))
	}
}
