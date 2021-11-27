use math::{
	vec2,
	Vector2,
};
use rapier2d::prelude::*;
use serde::{
	Deserialize,
	Serialize,
};
use sync::prelude::*;

use super::{
	Entity,
	World,
};

pub struct PhysicsWorld {
	// Sets of things in the physics worlds
	bodies: RigidBodySet,
	colliders: ColliderSet,
	joints: JointSet,

	// World parameters to tune
	gravity: Vector2,
	integration_parameters: IntegrationParameters,

	physics_pipeline: PhysicsPipeline,
	islands: IslandManager,
	broad_phase: BroadPhase,
	narrow_phase: NarrowPhase,
	ccd_solver: CCDSolver,

	step_accum: f32,
}

impl PhysicsWorld {
	pub fn new() -> Self {
		Self {
			bodies: RigidBodySet::new(),
			colliders: ColliderSet::new(),
			joints: JointSet::new(),

			gravity: vec2!(0.0, -9.8),
			integration_parameters: IntegrationParameters::default(),

			physics_pipeline: PhysicsPipeline::new(),
			islands: IslandManager::new(),
			broad_phase: BroadPhase::new(),
			narrow_phase: NarrowPhase::new(),
			ccd_solver: CCDSolver::new(),

			step_accum: 0.0,
		}
	}

	// TODO: Handle large dt properly
	pub fn step(&mut self, dt: f32) {
		let PhysicsWorld {
			bodies,
			colliders,
			joints,
			gravity,
			integration_parameters,
			physics_pipeline,
			islands,
			broad_phase,
			narrow_phase,
			ccd_solver,
			step_accum,
		} = self;

		*step_accum += dt;
		if *step_accum > integration_parameters.dt {
			*step_accum = 0.0;
		} else {
			return;
		}

		let gravity = vector![gravity.x, gravity.y];

		let hooks = ();
		let events = ();

		physics_pipeline.step(
			&gravity,
			integration_parameters,
			islands,
			broad_phase,
			narrow_phase,
			bodies,
			colliders,
			joints,
			ccd_solver,
			&hooks,
			&events,
		)
	}
}

impl Default for PhysicsWorld {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct BoxCollider {
	#[serde(skip)]
	handle: Option<ColliderHandle>,
	size: Vector2,
}

impl BoxCollider {
	pub fn new(size: Vector2) -> Self {
		Self { handle: None, size }
	}
}

pub fn on_box_collider_added(
	world: &World,
	entity: &Entity,
) -> Box<dyn Future<Output = ()> + Unpin> {
	let future = async {};
}
