use math::Vector3;
use rapier3d::prelude::*;

use serde::{
	self,
	Deserialize,
	Serialize,
};

pub mod rigid_body;

use std::sync::RwLock;

#[derive(Serialize, Deserialize)]
struct PhysicsStateInner {
	gravity: Vector3,
	integration_parameters: IntegrationParameters,
	#[serde(skip)]
	physics_pipeline: PhysicsPipeline,
	rigid_body_set: RigidBodySet,
	collider_set: ColliderSet,
	island_manager: IslandManager,
	broad_phase: BroadPhase,
	narrow_phase: NarrowPhase,
	joint_set: JointSet,
	ccd_solver: CCDSolver,
	query_pipeline: QueryPipeline,
}

pub struct PhysicsState(RwLock<PhysicsStateInner>);

impl PhysicsState {
	pub fn new(gravity: impl Into<Vector3>) -> Self {
		let integration_parameters = IntegrationParameters::default();
		let physics_pipeline = PhysicsPipeline::new();
		let island_manager = IslandManager::new();
		let broad_phase = BroadPhase::new();
		let narrow_phase = NarrowPhase::new();
		let joint_set = JointSet::new();
		let ccd_solver = CCDSolver::new();
		let rigid_body_set = RigidBodySet::new();
		let collider_set = ColliderSet::new();
		let query_pipeline = QueryPipeline::new();

		Self(RwLock::new(PhysicsStateInner {
			gravity: gravity.into(),
			integration_parameters,
			physics_pipeline,
			rigid_body_set,
			collider_set,
			island_manager,
			broad_phase,
			narrow_phase,
			joint_set,
			ccd_solver,
			query_pipeline,
		}))
	}

	pub fn simulate(&self) {
		let mut write = self.0.write().unwrap();

		let PhysicsStateInner {
			gravity,
			integration_parameters,
			physics_pipeline,
			island_manager,
			broad_phase,
			narrow_phase,
			joint_set,
			ccd_solver,
			rigid_body_set,
			collider_set,
			query_pipeline,
		} = &mut *write;

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
	}
}
