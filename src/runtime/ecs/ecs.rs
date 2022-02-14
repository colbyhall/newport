#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(const_type_name)]
#![feature(trait_alias)]
#![feature(wrapping_int_impl)]

use serde::{
	Deserialize,
	Serialize,
};

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Named {
	pub name: String,
}

impl Named {
	pub fn new(name: impl ToString) -> Self {
		Self {
			name: name.to_string(),
		}
	}
}

impl Component for Named {}

pub struct Ecs;
impl Module for Ecs {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<ResourceManager>()
			.register(SceneImporter::variant(&["scene"]))
			.register(Named::variant())
	}
}
