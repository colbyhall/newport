use ecs::*;
use engine::Builder;
use math::*;
use serde::{
	Deserialize,
	Serialize,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Transform {
	pub location: Vec2,
	pub rotation: f32,
	pub scale: Vec2,
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			location: Vec2::ZERO,
			rotation: 0.0,
			scale: Vec2::ONE,
		}
	}
}

pub fn register_components(builder: Builder) -> Builder {
	builder.register(Transform::variant())
}
