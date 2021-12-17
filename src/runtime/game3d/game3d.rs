mod editor;
mod render;
pub(crate) use {
	editor::*,
	render::*,
};

use {
	ecs::{
		Component,
		Ecs,
		ScheduleBlock,
		Transform,
		World,
	},
	engine::{
		Builder,
		Engine,
		Module,
	},
	graphics::Graphics,
	math::*,
	resources::Handle,
	std::cell::UnsafeCell,
	sync::join,
};

pub struct Game {
	world: World,
	renderer: Renderer,

	viewport: UnsafeCell<Vec2>,
}
impl Module for Game {
	fn new() -> Self {
		let world = World::new(ScheduleBlock::new());
		sync::block_on(async {
			let mut transforms = world.write::<Transform>().await;
			let mut filters = world.write::<MeshFilter>().await;
			let mut cameras = world.write::<Camera>().await;

			let pipeline =
				Handle::find_or_load("{D0FAF8AC-0650-48D1-AAC2-E1C01E1C93FC}").unwrap_or_default();
			world
				.spawn()
				.await
				.with(Transform::default(), &mut transforms)
				.with(Camera::default(), &mut cameras)
				.finish();

			world
				.spawn()
				.await
				.with(
					Transform {
						location: Point3::new(5.0, 0.0, -1.0),
						..Default::default()
					},
					&mut transforms,
				)
				.with(
					MeshFilter {
						mesh: Handle::find_or_load("{03383b92-566f-4036-aeb4-850b61685ea6}")
							.unwrap_or_default(),
						pipeline: pipeline.clone(),
					},
					&mut filters,
				)
				.finish();
		});

		let window = Engine::window().unwrap();
		let viewport = window.inner_size();
		let viewport = Vec2::new(viewport.width as f32, viewport.height as f32);

		Self {
			world,
			renderer: Renderer::new(),

			viewport: UnsafeCell::new(viewport),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<Ecs>()
			.module::<Editor>()
			.register(Camera::variant())
			.register(MeshFilter::variant())
			.tick(|delta_time| {
				let Game {
					world,
					renderer,
					viewport,
					..
				} = Engine::module().unwrap();

				let viewport = {
					let viewport = viewport.get();
					unsafe { *viewport }
				};

				Engine::wait_on(async {
					let simulation = async {
						world.step(delta_time).await;

						let scene = DrawList::build(world, viewport).await;
						renderer.push_scene(scene);
					};
					let render = renderer.render_scene();

					join!(simulation, render)
				});
				renderer.advance_frame();
			})
	}
}
