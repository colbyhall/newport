#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(const_type_name)]

mod component;
mod entity;
mod query;
mod scene;
mod schedule;
mod system;
mod world;
// mod physics;

pub use {
	component::*,
	entity::*,
	query::*,
	scene::*,
	schedule::*,
	system::*,
	world::*,
};

use engine::{
	Builder,
	Module,
};
use resources::Importer;

pub struct Ecs;

impl Module for Ecs {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.register(SceneImporter::variant(&["scene"]))
	}
}
