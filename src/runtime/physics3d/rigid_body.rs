use {
	ecs::Component,
	math::Vec3,
	rapier3d::prelude::*,
	serde::{
		Deserialize,
		Serialize,
	},
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RigidBody {
	pub(crate) handle: Option<RigidBodyHandle>,

	#[serde(flatten)]
	pub(crate) description: RigidBodyDescription,
}

impl RigidBody {
	pub fn builder(variant: RigidBodyVariant) -> RigidBodyBuilder {
		RigidBodyBuilder {
			variant,
			..Default::default()
		}
	}
}

impl Component for RigidBody {}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyVariant {
	Dynamic,
	Static,
	Kinematic,
}

pub type RigidBodyBuilder = RigidBodyDescription;
#[derive(Serialize, Deserialize, Clone)]
pub struct RigidBodyDescription {
	pub variant: RigidBodyVariant,

	pub linear_velocity: Vec3,
	pub angular_velocity: Vec3,
	pub gravity_scale: f32,
	pub linear_damping: f32,
	pub angular_damping: f32,
	pub can_sleep: bool,
	pub sleeping: bool,
	pub ccd_enabled: bool,
}

impl RigidBodyDescription {
	pub fn build(self) -> RigidBody {
		RigidBody {
			handle: None,

			description: self,
		}
	}
}

impl Default for RigidBodyDescription {
	fn default() -> Self {
		Self {
			variant: RigidBodyVariant::Static,

			linear_velocity: Vec3::ZERO,
			angular_velocity: Vec3::ZERO,
			gravity_scale: 1.0,
			linear_damping: 0.0,
			angular_damping: 0.0,
			can_sleep: true,
			sleeping: false,
			ccd_enabled: false,
		}
	}
}
