mod render;
pub(crate) use render::*;

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
	gpu::{
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
	},
	graphics::Graphics,
	math::*,
	resources::Handle,
	sync::join,
};

pub struct Game {
	world: World,
	renderer: Renderer,
	present_pipeline: Handle<GraphicsPipeline>,
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
		Self {
			world,
			renderer: Renderer::new(),
			present_pipeline: Handle::find_or_load("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<Ecs>()
			.register(Camera::variant())
			.register(MeshFilter::variant())
			.tick(|dt| {
				let Game {
					world, renderer, ..
				} = Engine::module().unwrap();
				Engine::wait_on(async {
					let simulation = async {
						world.step(dt).await;

						let window = Engine::window().unwrap();
						let viewport = window.inner_size();
						let viewport = Vec2::new(viewport.width as f32, viewport.height as f32);

						let scene = DrawList::build(world, viewport).await;
						renderer.push_scene(scene);
					};
					let render = renderer.render_scene();

					join!(simulation, render)
				});
				renderer.advance_frame();
			})
			.display(|| {
				let game: &Game = Engine::module().unwrap();

				let device = Gpu::device();
				let backbuffer = device
					.acquire_backbuffer()
					.expect("Swapchain failed to find a back buffer");

				let pipeline = game.present_pipeline.read();
				let receipt = match game.renderer.to_display() {
					Some(scene) => GraphicsRecorder::new()
						.render_pass(&[&backbuffer], |ctx| {
							ctx.clear_color(Color::BLACK)
								.bind_pipeline(&pipeline)
								.bind_texture("texture", &scene.diffuse_buffer)
								.draw(3, 0)
						})
						.resource_barrier_texture(
							&backbuffer,
							gpu::Layout::ColorAttachment,
							gpu::Layout::Present,
						)
						.submit(),
					None => GraphicsRecorder::new()
						.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::BLACK))
						.resource_barrier_texture(
							&backbuffer,
							gpu::Layout::ColorAttachment,
							gpu::Layout::Present,
						)
						.submit(),
				};

				device.display(&[receipt]);
			})
	}
}
