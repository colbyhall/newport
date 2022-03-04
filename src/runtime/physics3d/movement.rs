use {
	crate::{
		Collider,
		Movement,
		PhysicsManager,
		RigidBody,
	},
	ecs::{
		Component,
		Query,
		System,
		World,
	},
	game::Transform,
	math::Vec3,
	serde::{
		Deserialize,
		Serialize,
	},
};

#[derive(Serialize, Deserialize, Clone)]
pub enum BipedMovementMode {
	Falling,
	Walking,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BipedMovement {
	pub input: Vec3,
	pub jump_pressed: bool,

	pub velocity: Vec3,
}

impl Default for BipedMovement {
	fn default() -> Self {
		Self {
			input: Vec3::ZERO,
			jump_pressed: false,

			velocity: Vec3::ZERO,
		}
	}
}

impl Component for BipedMovement {}

#[derive(Clone)]
pub struct BipedMovementSystem;
impl System for BipedMovementSystem {
	fn run(&self, world: &World, dt: f32) {
		let mut physics = world.write::<PhysicsManager>();
		let mut physics = physics.get_mut_or_default(world.singleton);

		let transforms = world.write::<Transform>();
		let biped_movements = world.write::<BipedMovement>();
		let colliders = world.read::<Collider>();
		let rigid_bodies = world.read::<RigidBody>();

		let entities = Query::new()
			.write(&transforms)
			.write(&biped_movements)
			.read(&colliders)
			.read(&rigid_bodies)
			.execute(world);

		for e in entities.iter().cloned() {
			let mut transform = transforms.get_mut(e).unwrap();
			let biped_movement = biped_movements.get_mut(e).unwrap();
			let collider = colliders.get(e).unwrap();
			let rigid_body = rigid_bodies.get(e).unwrap();

			let acceleration = biped_movement.input * 5.0 + Vec3::UP * -9.8;

			let movement = Movement::new(acceleration * dt, transform.local_rotation());
			physics.move_rigid_body(
				movement,
				e,
				&rigid_body,
				&collider,
				&mut transform,
				&transforms,
			);
		}
	}
}
