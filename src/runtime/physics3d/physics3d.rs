mod collider;
mod movement;
mod query;
mod rigid_body;

use {
	ecs::{
		Component,
		Entity,
		Query as EcsQuery,
		System,
		World,
		WriteStorage,
	},
	engine::{
		Builder,
		Module,
	},
	game::Transform,
	math::{
		Quat,
		Vec3,
	},
	query::Query,
	rapier3d::{
		na::{
			Isometry3,
			Quaternion,
			Translation3,
			UnitQuaternion,
		},
		prelude::{
			nalgebra,

			vector,
			BroadPhase,
			CCDSolver,
			ColliderSet,
			IntegrationParameters,
			IslandManager,
			JointSet,
			NarrowPhase,
			PhysicsPipeline,
			QueryPipeline,
			RigidBodySet,
			RigidBodyType,
		},
	},
	serde::{
		Deserialize,
		Serialize,
	},
	std::ops::DerefMut,
};

pub use {
	collider::{
		Collider,
		Shape,
	},
	movement::{
		BipedMovement,
		BipedMovementMode,
		BipedMovementSystem,
	},
	query::{
		Filter,
		RayCast,
		RayCastHit,
		ShapeCast,
		ShapeCastHit,
		ShapeCastStatus,
	},
	rigid_body::{
		RigidBody,
		RigidBodyVariant,
	},
};

pub struct Physics;
impl Module for Physics {
	fn new() -> Self {
		Physics
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<game::Game>()
			.register(PhysicsManager::variant())
			.register(Collider::variant())
			.register(RigidBody::variant())
			.register(BipedMovement::variant())
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Movement {
	pub delta: Vec3,
	pub rotation: Quat,
	pub sweep: bool,
}

impl Movement {
	pub fn new(delta: impl Into<Vec3>, rotation: Quat) -> Self {
		Self {
			delta: delta.into(),
			rotation,
			sweep: true,
		}
	}

	pub fn sweep(mut self, sweep: bool) -> Self {
		self.sweep = sweep;
		self
	}
}

#[derive(Serialize, Deserialize)]
pub struct PhysicsManager {
	integration_parameters: IntegrationParameters,
	#[serde(skip, default = "PhysicsPipeline::new")]
	physics_pipeline: PhysicsPipeline,
	island_manager: IslandManager,
	broad_phase: BroadPhase,
	narrow_phase: NarrowPhase,
	joint_set: JointSet,
	ccd_solver: CCDSolver,
	rigid_body_set: RigidBodySet,
	collider_set: ColliderSet,
	query_pipeline: QueryPipeline,

	gravity: Vec3,
	timer: f32,
}

impl PhysicsManager {
	pub fn new() -> Self {
		Self {
			integration_parameters: IntegrationParameters::default(),
			physics_pipeline: PhysicsPipeline::new(),
			island_manager: IslandManager::new(),
			broad_phase: BroadPhase::new(),
			narrow_phase: NarrowPhase::new(),
			joint_set: JointSet::new(),
			ccd_solver: CCDSolver::new(),
			rigid_body_set: RigidBodySet::new(),
			collider_set: ColliderSet::new(),
			query_pipeline: QueryPipeline::new(),

			gravity: Vec3::new(0.0, 0.0, -9.8),
			timer: 0.0,
		}
	}

	#[must_use]
	pub fn single_cast<T: Query>(&self, cast: T, filter: Filter) -> Option<T::Hit> {
		cast.single(filter, self)
	}

	pub fn move_rigid_body(
		&mut self,
		movement: Movement,

		entity: Entity,
		rigid_body: &RigidBody,
		collider: &Collider,
		transform: &mut Transform,
		transforms: &WriteStorage<Transform>,
	) -> Option<ShapeCastHit> {
		let location = transform.local_location();

		let result = if movement.sweep {
			let start = location;
			let end = start + movement.delta;
			let cast = ShapeCast::new(start, end, movement.rotation, collider.shape());
			let filter = Filter {
				ignore: vec![entity],
			};
			let hit = self.single_cast(cast, filter);
			if let Some(hit) = &hit {
				match hit.status {
					ShapeCastStatus::Penetrating => {
						transform.set_local_location_and_rotation(
							start,
							movement.rotation,
							transforms,
						);
					}
					ShapeCastStatus::Success {
						origin_at_impact, ..
					} => {
						transform.set_local_location_and_rotation(
							origin_at_impact,
							movement.rotation,
							transforms,
						);
					}
				}
			} else {
				transform.set_local_location_and_rotation(end, movement.rotation, transforms);
			}
			hit
		} else {
			transform.set_local_location_and_rotation(
				location + movement.delta,
				movement.rotation,
				transforms,
			);
			None
		};
		transform.set_changed(false);

		if let Some(rigid_body_handle) = rigid_body.handle {
			let rigid_body = self.rigid_body_set.get_mut(rigid_body_handle).unwrap();
			let location = transform.local_location();
			let rotation = transform.local_rotation();
			rigid_body.set_next_kinematic_position(Isometry3::from_parts(
				Translation3::new(location.x, location.y, location.z),
				UnitQuaternion::from_quaternion(Quaternion::new(
					rotation.w, rotation.x, rotation.y, rotation.z,
				)),
			));
		}

		result
	}
}

impl Component for PhysicsManager {}

impl Default for PhysicsManager {
	fn default() -> Self {
		Self::new()
	}
}

impl Clone for PhysicsManager {
	fn clone(&self) -> Self {
		Self {
			integration_parameters: self.integration_parameters,
			physics_pipeline: PhysicsPipeline::new(),
			island_manager: self.island_manager.clone(),
			broad_phase: self.broad_phase.clone(),
			narrow_phase: self.narrow_phase.clone(),
			joint_set: self.joint_set.clone(),
			ccd_solver: self.ccd_solver.clone(),
			rigid_body_set: self.rigid_body_set.clone(),
			collider_set: self.collider_set.clone(),
			query_pipeline: self.query_pipeline.clone(),

			gravity: self.gravity,
			timer: self.timer,
		}
	}
}
#[derive(Clone)]
pub struct PhysicsSystem;
impl System for PhysicsSystem {
	fn run(&self, world: &World, dt: f32) {
		let mut physics_managers = world.write::<PhysicsManager>();
		let mut physics_manager = physics_managers.get_mut_or_default(world.singleton);

		let PhysicsManager {
			integration_parameters,
			physics_pipeline,
			island_manager,
			broad_phase,
			narrow_phase,
			joint_set,
			ccd_solver,
			rigid_body_set,
			collider_set,
			gravity,
			timer,
			query_pipeline,
		} = physics_manager.deref_mut();

		let transforms = world.write::<Transform>();
		let colliders = world.write::<Collider>();
		let rigid_bodies = world.write::<RigidBody>();

		let entities = EcsQuery::new()
			.write(&transforms)
			.write(&colliders)
			// .write(&rigid_bodies)
			.execute(world);

		// Register all unknown colliders and rigid bodies
		// FIXME: Update any transforms if they have changed
		for e in entities.iter().copied() {
			let transform = transforms.get(e).unwrap();
			let mut collider = colliders.get_mut(e).unwrap();
			if collider.handle.is_none() {
				let rapier_collider = match collider.description.shape {
					Shape::Cube { half_extents } => rapier3d::prelude::ColliderBuilder::cuboid(
						half_extents.x,
						half_extents.y,
						half_extents.z,
					),
					Shape::Capsule {
						half_height,
						radius,
					} => rapier3d::prelude::ColliderBuilder::capsule_z(half_height, radius),
					_ => unimplemented!(),
				}
				.sensor(collider.description.sensor)
				.user_data(e.into())
				.build();
				if let Some(mut rigid_body) = rigid_bodies.get_mut(e) {
					if rigid_body.handle.is_none() {
						let body_type = match rigid_body.description.variant {
							RigidBodyVariant::Dynamic => RigidBodyType::Dynamic,
							RigidBodyVariant::Static => RigidBodyType::Static,
							RigidBodyVariant::Kinematic => RigidBodyType::KinematicPositionBased,
						};

						let location = transform.local_location();
						let rapier_rigid_body = rapier3d::prelude::RigidBodyBuilder::new(body_type)
							.translation(vector![location.x, location.y, location.z])
							.user_data(e.into())
							.build();
						rigid_body.handle = Some(rigid_body_set.insert(rapier_rigid_body));
					}
					let rigid_body_handle = rigid_body
						.handle
						.expect("RigidBody should have been created by now");

					let collider_handle = collider_set.insert_with_parent(
						rapier_collider,
						rigid_body_handle,
						rigid_body_set,
					);
					collider.handle = Some(collider_handle);
				} else {
					collider.handle = Some(collider_set.insert(rapier_collider));
				}
			} else if transform.changed() {
				// FIXME: Only do this when we're doing the physics step
				if let Some(rigid_body) = rigid_bodies.get(e) {
					if rigid_body.description.variant == RigidBodyVariant::Kinematic {
						let handle = rigid_body.handle.unwrap();
						let rigid_body = rigid_body_set.get_mut(handle).unwrap();

						let location = transform.local_location();
						let rotation = transform.local_rotation();
						// FIXME: Do Scale

						let mut next_position =
							Isometry3::translation(location.x, location.y, location.z);
						next_position.rotation = UnitQuaternion::from_quaternion(Quaternion::new(
							rotation.w, rotation.x, rotation.y, rotation.z,
						));

						rigid_body.set_next_kinematic_position(next_position);
					} else {
						// FIXME: Print out warning or crash?
					}
				}
			}
		}

		const TIME_STEP: f32 = 1.0 / 60.0;
		*timer += dt;
		if *timer >= TIME_STEP {
			*timer -= TIME_STEP;
			let physics_hooks = ();
			let event_handler = ();

			let gravity = vector![gravity.x, gravity.y, gravity.z];

			physics_pipeline.step(
				&gravity,
				integration_parameters,
				island_manager,
				broad_phase,
				narrow_phase,
				rigid_body_set,
				collider_set,
				joint_set,
				ccd_solver,
				&physics_hooks,
				&event_handler,
			);

			query_pipeline.update(island_manager, rigid_body_set, collider_set);

			// Iterate through every entity with a rigid body and update their locations and rotations.
			// FIXME: Only update the entities that actually changed
			for e in entities.iter().copied() {
				if let Some(rigid_body) = rigid_bodies.get_mut(e) {
					let mut transform = transforms.get_mut(e).unwrap();
					let rigid_body = rigid_body_set
						.get(rigid_body.handle.unwrap())
						.expect("Should be registered");

					let location = rigid_body.translation();
					let rotation = rigid_body.rotation();
					transform.set_local_location_and_rotation(
						[location[0], location[1], location[2]],
						Quat {
							x: rotation.i,
							y: rotation.j,
							z: rotation.k,
							w: rotation.w,
						},
						&transforms,
					);
					transform.set_changed(false);
				}
			}
		}
	}
}
