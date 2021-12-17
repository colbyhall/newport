#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(const_type_name)]
#![feature(trait_alias)]
#![feature(wrapping_int_impl)]

use math::*;
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

pub struct Ecs;
impl Module for Ecs {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<ResourceManager>()
			.register(SceneImporter::variant(&["scene"]))
			.register(Transform::variant())
	}
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Transform {
	pub location: Point3,
	pub rotation: Quaternion,
	pub scale: Vec3,
}

impl Transform {
	pub fn to_mat4(&self) -> Mat4 {
		// TODO: Do this without mat4 multiplication
		Mat4::rotate(self.rotation) * Mat4::translate(self.location)
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			location: Point3::ZERO,
			rotation: Quaternion::IDENTITY,
			scale: Vec3::ONE,
		}
	}
}
