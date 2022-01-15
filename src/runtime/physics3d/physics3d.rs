use {
	ecs::{
		Component,
		System,
		World,
	},
	engine::{
		Builder,
		Module,
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

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<game3d::Game>()
			.register(PhysicsManager::variant())
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
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct Collider {
	handle: ColliderHandle,
}

#[derive(Serialize, Deserialize)]
pub struct RigidBody {
	handle: RigidBodyHandle,
}

#[derive(Clone)]
pub struct PhysicsSystem;

impl System for PhysicsSystem {
	fn run(&self, world: &World, dt: f32) {
		let mut physics_states = world.write::<PhysicsManager>();
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
		} = physics_states.get_mut(&world.singleton).unwrap();

		let physics_hooks = ();
		let event_handler = ();

		let gravity = vector![0.0, 0.0, -9.81];

		integration_parameters.dt = dt as _; // TODO: use the type here

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
	}
}
