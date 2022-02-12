use ecs::ScheduleBlock;

use {
	ecs::Named,
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	game3d::*,
	input::InputSystem,
	math::Vec3,
	physics3d::*,
	resources::Handle,
};

pub struct Orchard;
impl Module for Orchard {
	fn new() -> Self {
		let game: &Game = Engine::module().unwrap();
		{
			let mut schedule = game.schedule.lock().unwrap();
			*schedule = ScheduleBlock::new()
				.system(InputSystem)
				.system(DebugSystem)
				.system(PhysicsSystem)
				.system(DobblerSystem)
				.system(EditorCameraSystem);
		}

		let world = &game.world;

		let mut transforms = world.write::<Transform>();
		let mut filters = world.write::<MeshFilter>();
		let mut cameras = world.write::<Camera>();
		let mut names = world.write::<Named>();
		let mut camera_controllers = world.write::<EditorCameraController>();
		let mut colliders = world.write::<Collider>();
		let mut rigid_bodies = world.write::<RigidBody>();

		let pipeline = Handle::find_or_load("{D0FAF8AC-0650-48D1-AAC2-E1C01E1C93FC}").unwrap();

		world
			.spawn(world.persistent)
			.with(Named::new("Camera"), &mut names)
			.with(
				Transform::builder().location([0.0, 0.0, 5.0]).finish(),
				&mut transforms,
			)
			.with(Camera::default(), &mut cameras)
			.with(EditorCameraController::default(), &mut camera_controllers)
			.finish();

		for i in 0..10 {
			let z = (i * 2) as f32;
			let y = i as f32 / 2.0;
			world
				.spawn(world.persistent)
				.with(Named::new("Block"), &mut names)
				.with(
					Transform::builder()
						.location(Vec3::new(5.0, y, z + 5.0))
						.finish(),
					&mut transforms,
				)
				.with(
					MeshFilter {
						mesh: Handle::find_or_load("{03383b92-566f-4036-aeb4-850b61685ea6}")
							.unwrap(),
						pipeline: pipeline.clone(),
					},
					&mut filters,
				)
				.with(
					Collider::builder(Shape::cube(Vec3::ONE / 2.0)).build(),
					&mut colliders,
				)
				.with(
					RigidBody::builder(RigidBodyVariant::Dynamic).build(),
					&mut rigid_bodies,
				)
				.finish();
		}

		let floor_size = Vec3::new(10000.0, 10000.0, 0.1);
		world
			.spawn(world.persistent)
			.with(Named::new("Floor"), &mut names)
			.with(
				Transform::builder().scale(floor_size).finish(),
				&mut transforms,
			)
			.with(
				MeshFilter {
					mesh: Handle::find_or_load("{03383b92-566f-4036-aeb4-850b61685ea6}").unwrap(),
					pipeline,
				},
				&mut filters,
			)
			.with(
				Collider::builder(Shape::cube(floor_size / 2.0)).build(),
				&mut colliders,
			)
			// .with(
			// 	RigidBody::builder(RigidBodyVariant::Static).build(),
			// 	&mut rigid_bodies,
			// )
			.finish();

		Self
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder.module::<Game>().module::<Physics>()
	}
}

define_run_module!(Orchard, "Orchard");
