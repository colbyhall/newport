#![feature(trait_alias)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![feature(const_type_name)]
#![allow(arithmetic_overflow)]

mod component;
mod entity;
mod query;
mod scene;
mod schedule;
mod system;
mod world;

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

pub struct Ecs;

impl Module for Ecs {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.register(asset::Variant::new::<SceneImporter>(&["scene"]))
	}
}
