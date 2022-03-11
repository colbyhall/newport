use {
	crate::{
		Collider,
		Movement,
		PhysicsManager,
		RigidBody,
		ShapeCastStatus,
	},
	ecs::{
		Component,
		Query,
		System,
		World,
	},
	engine::error,
	game::Transform,
	math::Vec3,
	serde::{
		Deserialize,
		Serialize,
	},
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BipedMovementMode {
	Falling,
	Walking,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BipedMovement {
	pub input: Vec3,
	pub jump_pressed: bool,

	pub velocity: Vec3,
	pub mode: BipedMovementMode,
}

impl Default for BipedMovement {
	fn default() -> Self {
		Self {
			input: Vec3::ZERO,
			jump_pressed: false,

			velocity: Vec3::ZERO,
			mode: BipedMovementMode::Falling,
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
			let mut biped_movement = biped_movements.get_mut(e).unwrap();
			let collider = colliders.get(e).unwrap();
			let rigid_body = rigid_bodies.get(e).unwrap();

			match biped_movement.mode {
				BipedMovementMode::Falling => {
					let acceleration = Vec3::UP * -9.8;

					let movement = Movement::new(acceleration * dt, transform.local_rotation());
					let hit = physics.move_rigid_body(
						movement,
						e,
						&rigid_body,
						&collider,
						&mut transform,
						&transforms,
					);

					if hit.is_some() {
						biped_movement.mode = BipedMovementMode::Walking;
					}
				}
				BipedMovementMode::Walking => {
					let acceleration = biped_movement.input * 5.0;

					let movement = Movement::new(acceleration * dt, transform.local_rotation());
					let hit = physics.move_rigid_body(
						movement,
						e,
						&rigid_body,
						&collider,
						&mut transform,
						&transforms,
					);

					if let Some(hit) = hit {
						if matches!(hit.status, ShapeCastStatus::Penetrating) {
							error!("Penetrating {:?}", hit.entity);
						}
					}
				}
			}
		}
	}
}
