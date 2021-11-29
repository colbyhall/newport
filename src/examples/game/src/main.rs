mod components;

use {
	components::*,
	ecs::*,
	engine::{define_run_module, Builder, Engine, Module},
	gpu::*,
	graphics::*,
	math::*,
	resources::*,
};

struct Game {
	schedule: Schedule,
	world: World,

	style: PainterStyle,
	draw_pipeline: Handle<GraphicsPipeline>,
}

impl Module for Game {
	fn new() -> Self {
		let mut style = PainterStyle::default();
		style.line_width = 20.0;

		let schedule = Schedule::builder().spawn();
		let world = World::default();

		Self {
			schedule,
			world,

			style,
			draw_pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		register_components(builder)
			.module::<Graphics>()
			.module::<ResourceManager>()
			.tick(|dt| {
				let game: &Game = Engine::module().unwrap();
				Engine::wait_on(game.schedule.execute(&game.world, dt));
			})
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device
					.acquire_backbuffer()
					.expect("Swapchain failed to find a back buffer");

				// let game: &Game = Engine::module().unwrap();

				// #[allow(dead_code)]
				// struct Imports {
				// 	view: Matrix4,
				// }

				// let scale = Engine::window().unwrap().scale_factor() as f32;
				// let viewport = vec2!(backbuffer.width() as f32, backbuffer.height() as f32) / scale;

				// let proj = Matrix4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
				// let view =
				// 	Matrix4::translate(Vector3::new(-viewport.x / 2.0, -viewport.y / 2.0, 0.0));

				// let imports =
				// 	Buffer::new(BufferUsage::CONSTANTS, gpu::MemoryType::HostVisible, 1).unwrap();
				// imports.copy_to(&[Imports { view: proj * view }]).unwrap();

				// let pipeline = game.draw_pipeline.read();

				let receipt = GraphicsRecorder::new()
					.resource_barrier_texture(
						&backbuffer,
						Layout::Undefined,
						Layout::ColorAttachment,
					)
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
						// .bind_pipeline(&pipeline)
						// .bind_vertex_buffer(&vertex_buffer)
						// .bind_index_buffer(&index_buffer)
						// .bind_constants("imports", &imports, 0)
						// .draw_indexed(index_buffer.len(), 0)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit();

				device.display(&[receipt]);
			})
	}
}

define_run_module!(Game, "Game Example");
