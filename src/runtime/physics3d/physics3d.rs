use game3d::DebugManager;

use {
	ecs::{
		Component,
		Query,
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

			gravity: Vec3::new(0.0, 0.0, -9.8),
			timer: 0.0,
		}
	}
}

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
		// Lazy load the physics state
		let mut physics_managers = world.write::<PhysicsManager>();
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
		} = match physics_managers.get_mut(world.singleton) {
			Some(c) => c,
			None => {
				world.insert(
					&mut physics_managers,
					world.singleton,
					PhysicsManager::default(),
				);
				physics_managers.get_mut(world.singleton).unwrap()
			}
		};

		let mut transforms = world.write::<Transform>();
		let mut colliders = world.write::<Collider>();
		let mut rigid_bodies = world.write::<RigidBody>();

		let entities = Query::new()
			.write(&transforms)
			.write(&colliders)
			// .write(&rigid_bodies)
			.execute(world);

		// Register all unknown colliders and rigid bodies
		for e in entities.iter().copied() {
			let transform = transforms.get(e).unwrap();
			let collider = colliders.get_mut(e).unwrap();
			if collider.handle.is_none() {
				let rapier_collider = match collider.description.shape {
					Shape::Cube { half_extents } => rapier3d::prelude::ColliderBuilder::cuboid(
						half_extents.x,
						half_extents.y,
						half_extents.z,
					),
					_ => unimplemented!(),
				}
				.sensor(collider.description.sensor)
				.build();
				if let Some(rigid_body) = rigid_bodies.get_mut(e) {
					if rigid_body.handle.is_none() {
						let body_type = match rigid_body.description.variant {
							RigidBodyVariant::Dynamic => RigidBodyType::Dynamic,
							RigidBodyVariant::Static => RigidBodyType::Static,
							RigidBodyVariant::Kinematic => RigidBodyType::KinematicPositionBased,
						};

						let location = transform.location();
						let rapier_rigid_body = rapier3d::prelude::RigidBodyBuilder::new(body_type)
							.translation(vector![location.x, location.y, location.z])
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

			let mut debug_managers = world.write::<DebugManager>();
			let debug = debug_managers.get_mut(world.singleton).unwrap();
			for e in entities.iter().copied() {
				if let Some(rigid_body) = rigid_bodies.get_mut(e) {
					let transform = transforms.get_mut(e).unwrap();
					let collider = colliders.get(e).unwrap();
					let rigid_body = rigid_body_set
						.get(rigid_body.handle.unwrap())
						.expect("Should be registered");

					let location = rigid_body.translation();
					let rotation = rigid_body.rotation();
					transform.set_location([location[0], location[1], location[2]]);
					transform.set_rotation(Quat {
						x: rotation.i,
						y: rotation.j,
						z: rotation.k,
						w: rotation.w,
					});

					match collider.description.shape {
						Shape::Cube { half_extents } => {
							debug.draw_box(
								transform.location(),
								transform.rotation(),
								half_extents,
								1.0 / 60.0,
							);
						}
						_ => unimplemented!(),
					}
				}
			}
		}
	}
}
