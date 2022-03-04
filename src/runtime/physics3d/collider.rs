use {
	ecs::Component,
	math::Vec3,
	rapier3d::prelude::*,
	serde::{
		Deserialize,
		Serialize,
	},
};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Collider {
	pub(crate) handle: Option<ColliderHandle>,

	#[serde(flatten)]
	pub(crate) description: ColliderDescription,
}

impl Collider {
	pub fn builder(shape: Shape) -> ColliderBuilder {
		ColliderBuilder {
			shape,
			..Default::default()
		}
	}
}

impl Component for Collider {}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Shape {
	Sphere { radius: f32 },
	Cube { half_extents: Vec3 },
	Capsule { half_height: f32, radius: f32 },
}

impl Shape {
	pub const fn sphere(radius: f32) -> Self {
		Self::Sphere { radius }
	}

	pub fn cube(half_extents: impl Into<Vec3>) -> Self {
		Self::Cube {
			half_extents: half_extents.into(),
		}
	}

	pub const fn capsule(half_height: f32, radius: f32) -> Self {
		Self::Capsule {
			half_height,
			radius,
		}
	}
}

pub type ColliderBuilder = ColliderDescription;
#[derive(Serialize, Deserialize, Clone)]
pub struct ColliderDescription {
	pub enabled: bool,
	pub sensor: bool,
	pub shape: Shape,
	pub offset: Vec3,
}

impl ColliderBuilder {
	pub const fn enabled(mut self, enabled: bool) -> Self {
		self.enabled = enabled;
		self
	}

	pub const fn sensor(mut self, sensor: bool) -> Self {
		self.sensor = sensor;
		self
	}

	pub fn offset(mut self, offset: impl Into<Vec3>) -> Self {
		self.offset = offset.into();
		self
	}

	pub const fn build(self) -> Collider {
		Collider {
			handle: None,

			description: self,
		}
	}
}

impl Default for ColliderDescription {
	fn default() -> Self {
		Self {
			enabled: true,
			sensor: false,
			shape: Shape::cube(Vec3::ONE / 2.0),
			offset: Vec3::ZERO,
		}
	}
}
