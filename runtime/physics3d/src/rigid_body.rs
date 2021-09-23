pub enum RigidBodyVariant {
	Static,
	Dynamic,
	Kinematic,
}
use math::Vector3;
use rapier3d::prelude::RigidBodyHandle;

pub struct RigidBody {
	pub variant: RigidBodyVariant,
	pub handle: RigidBodyHandle,

	pub linear_velocity: Vector3,
	pub angulat_velocity: Vector3,

	pub gravity_scale: f32,

	pub liner_damping: f32,
	pub angular_damping: f32,

	pub can_sleep: bool,
	pub sleeping: bool,

	pub ccd_enabled: bool,

	pub density: f32,
}
