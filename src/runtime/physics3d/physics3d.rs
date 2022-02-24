use std::ops::DerefMut;

use ecs::Entity;
// use game3d::DebugManager;
use math::Point3;
use rapier3d::na::{
	Quaternion,
	UnitQuaternion,
};

use {
	ecs::{
		Component,
		Query as EcsQuery,
		System,
		World,
	},
	engine::{
		Builder,
		Module,
	},
	game3d::Transform,
	math::{
		Quat,
		Vec3,
	},
	rapier3d::prelude::*,
	serde::{
		Deserialize,
		Serialize,
	},
};

pub struct Physics;
impl Module for Physics {
	fn new() -> Self {
		Physics
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<game3d::Game>()
			.register(PhysicsManager::variant())
			.register(Collider::variant())
			.register(RigidBody::variant())
	}
}

pub trait SingleQuery {
	fn execute(self, manager: &PhysicsManager) -> Option<Query>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Raycast {
	pub origin: Point3,
	pub direction: Vec3,
	pub distance: f32,
	pub solid: bool, // If solid and is overlapping with collider at origin will hit
}

impl Raycast {
	pub fn new(origin: impl Into<Point3>, direction: impl Into<Vec3>, distance: f32) -> Self {
		Self {
			origin: origin.into(),
			direction: direction.into(),
			distance,
			solid: false,
		}
	}
}

impl SingleQuery for Raycast {
	fn execute(self, manager: &PhysicsManager) -> Option<Query> {
		let ray = Ray::new(
			point![self.origin.x, self.origin.y, self.origin.z],
			vector![self.direction.x, self.direction.y, self.direction.z],
		);

		let (collider, intersection) = manager.query_pipeline.cast_ray_and_get_normal(
			&manager.collider_set,
			&ray,
			self.distance,
			self.solid,
			InteractionGroups::all(),
			None,
		)?;

		let collider = manager.collider_set.get(collider).unwrap();
		let entity = collider.user_data.into();

		let impact = self.origin + self.direction * intersection.toi;
		let normal = Vec3::new(
			intersection.normal.x,
			intersection.normal.y,
			intersection.normal.z,
		);

		Some(Query {
			entity,

			impact,
			normal,
		})
	}
}

pub struct Query {
	pub entity: Entity,

	pub impact: Point3,
	pub normal: Vec3,
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

	pub fn single_cast(&self, cast: impl SingleQuery) -> Option<Query> {
		cast.execute(self)
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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Collider {
	handle: Option<ColliderHandle>,

	#[serde(flatten)]
	description: ColliderDescription,
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

#[derive(Serialize, Deserialize, Clone, Copy)]
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
	enabled: bool,
	sensor: bool,
	shape: Shape,
	offset: Vec3,
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RigidBody {
	handle: Option<RigidBodyHandle>,

	#[serde(flatten)]
	description: RigidBodyDescription,
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
	variant: RigidBodyVariant,

	linear_velocity: Vec3,
	angular_velocity: Vec3,
	gravity_scale: f32,
	linear_damping: f32,
	angular_damping: f32,
	can_sleep: bool,
	sleeping: bool,
	ccd_enabled: bool,
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
							Isometry::translation(location.x, location.y, location.z);
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

		*timer += dt;
		if *timer >= 1.0 / 60.0 {
			*timer = 0.0;
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

			// Grab the debug manager for later
			// let debug_managers = world.write::<DebugManager>();
			// let mut debug = debug_managers.get_mut(world.singleton).unwrap();

			// Iterate through every entity with a rigid body and update their locations and rotations.
			// TODO: Only update the entities that actually changed
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

					// let collider = colliders.get(e).unwrap();
					// match collider.description.shape {
					// 	Shape::Cube { half_extents } => {
					// 		debug.draw_box(
					// 			transform.local_location(),
					// 			transform.local_rotation(),
					// 			half_extents,
					// 			1.0 / 60.0,
					// 		);
					// 	}
					// 	Shape::Capsule {
					// 		half_height,
					// 		radius,
					// 	} => {
					// 		// FIXME: Draw an actual capsule whent that implementation is complete
					// 		debug.draw_box(
					// 			transform.local_location(),
					// 			transform.local_rotation(),
					// 			Vec3::new(radius, radius, half_height),
					// 			1.0 / 60.0,
					// 		);
					// 	}
					// 	_ => unimplemented!(),
					// }
				}
			}
		}
	}
}
