use ecs::{
	Schedule,
	World,
};

use asset::AssetRef;
use gpu::GraphicsPipeline;
use graphics::Mesh;

// use engine::Engine;

use crate::components::*;
use crate::systems::*;

use math::{ Vector3, Quaternion }; 

#[derive(Default)]
pub struct GameState {
	world: World,
	schedule: Schedule,
}

impl GameState {
	pub fn new() -> Self {
		// let mut schedules = Engine::register::<Schedule>();

		let cube_mesh: AssetRef<Mesh> =
			AssetRef::new("{03383b92-566f-4036-aeb4-850b61685ea6}").unwrap_or_default();
		let plane_mesh: AssetRef<Mesh> =
			AssetRef::new("{08FBEEC7-62E6-4834-8EA3-F92BF16D8364}").unwrap_or_default();
		let mesh_pipeline: AssetRef<GraphicsPipeline> =
			AssetRef::new("{D0FAF8AC-0650-48D1-AAC2-E1C01E1C93FC}").unwrap_or_default();

		let world = World::new();

		world
			.create()
			.with(Transform {
				location: Vector3::ZERO,
				rotation: Quaternion::IDENTITY,
				scale: Vector3::ONE,
			})
			.with(MeshRender {
				mesh: cube_mesh.clone(),
				pipeline: mesh_pipeline.clone(),
			})
			.spawn();

		world
			.create()
			.with(Transform {
				location: Vector3::ZERO,
				rotation: Quaternion::IDENTITY,
				scale: Vector3::new(1000.0, 1000.0, 1.0),
			})
			.with(MeshRender { mesh: plane_mesh, pipeline: mesh_pipeline.clone(), })
			.spawn();

		world
			.create()
			.with(Transform {
				location: Vector3::new(10.0, 3.0, -2.0),
				rotation: Quaternion::from_euler(Vector3::new(45.0, 120.0, 90.0)),
				scale: Vector3::ONE,
			})
			.with(MeshRender {
				mesh: cube_mesh.clone(),
				pipeline: mesh_pipeline.clone(),
			})
			.with(Spinner { speed: 36.0 })
			.spawn();

		world
			.create()
			.with(Transform {
				location: Vector3::new(-3.0, 10.0, 12.0),
				rotation: Quaternion::from_euler(Vector3::new(5.0, 20.0, 67.0)),
				scale: Vector3::ONE,
			})
			.with(MeshRender {
				mesh: cube_mesh.clone(),
				pipeline: mesh_pipeline.clone(),
			})
			.with(Spinner { speed: 72.0 })
			.spawn();

		world
			.create()
			.with(Transform {
				location: Vector3::new(30.0, -20.0, 0.0),
				rotation: Quaternion::from_euler(Vector3::new(23.0, 203.0, 67.0)),
				scale: Vector3::ONE,
			})
			.with(MeshRender { mesh: cube_mesh, pipeline: mesh_pipeline, })
			.with(Spinner { speed: 500.0 })
			.with(Scaler {
				speed: 0.5,
				max: 2.0,
				time: 0.0,
			})
			.spawn();

		world
			.create()
			.with(Transform {
				location: Vector3::new(-10.0, 0.0, 3.0),
				rotation: Quaternion::IDENTITY,
				scale: Vector3::ONE,
			})
			.with(Camera { fov: 90.0 })
			.with(CameraController::default())
			.spawn();

		let schedule = Schedule::builder()
			.single(Box::new(SpinDriver))
			.single(Box::new(ScaleDriver))
			.single(Box::new(CameraDriver))
			.spawn();

		Self {
			world,
			schedule,
		}
	}

	pub async fn simulate(&self, dt: f32) {
		let Self { world, schedule } = self;
		schedule.execute(world, dt).await
	}

	pub fn world(&self) -> &World {
		&self.world
	}
}
