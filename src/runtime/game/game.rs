use {
	draw2d::Draw2d,
	ecs::{
		Component,
		Ecs,
		ScheduleBlock,
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
	input::*,
	math::*,
	resources::{
		Handle,
		Importer,
		Resource,
	},
	std::sync::Mutex,
};

mod debug;
mod render;
mod transform;

pub(crate) use {
	debug::*,
	render::*,
};

pub use {
	debug::{
		DebugManager,
		DebugSystem,
	},
	render::{
		Camera,
		DirectionalLight,
		Mesh,
		MeshFilter,
	},
	transform::Transform,
};

pub struct Game {
	pub world: World,
	pub schedule: Mutex<ScheduleBlock>,
	renderer: Renderer,

	present_pipeline: Handle<GraphicsPipeline>,
}
impl Module for Game {
	fn new() -> Self {
		Self {
			world: World::new(),
			schedule: Mutex::new(ScheduleBlock::new()),
			renderer: Renderer::new(),

			#[cfg(not(feature = "editor"))]
			present_pipeline: Handle::find_or_load("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}")
				.unwrap(),
		}
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<Draw2d>()
			.module::<Ecs>()
			.module::<GameInput>()
			.register(Transform::variant())
			.register(Camera::variant())
			.register(Mesh::variant())
			.register(MeshGltfImporter::variant(&["gltf", "glb"]))
			.register(MeshFilter::variant())
			.register(DebugManager::variant())
			.register(DirectionalLight::variant())
			.tick(|delta_time| {
				let Game {
					world,
					renderer,
					schedule,
					..
				} = Engine::module().unwrap();

				let window = Engine::window().unwrap();
				let viewport = window.inner_size();
				let viewport = Vec2::new(viewport.width as f32, viewport.height as f32);

				{
					{
						let schedule = schedule.lock().unwrap();
						schedule.execute(world, delta_time);
						let scene = DrawList::build(world, viewport);
						renderer.push_scene(scene);
					}
					renderer.render_scene();
				};
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
								.set_pipeline(&pipeline)
								.set_texture("texture", &scene.diffuse_buffer)
								.draw(3, 0)
						})
						.texture_barrier(
							&backbuffer,
							gpu::Layout::ColorAttachment,
							gpu::Layout::Present,
						)
						.submit(),
					None => GraphicsRecorder::new()
						.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::BLACK))
						.texture_barrier(
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
