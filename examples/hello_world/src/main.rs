use {
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::*,
	graphics::*,

	math::Color,

	resources::*,
};

struct HelloWorld {
	style: PainterStyle,

	draw_pipeline: Handle<GraphicsPipeline>,
}

impl Module for HelloWorld {
	fn new() -> Self {
		Self {
			style: PainterStyle::default(),
			draw_pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<ResourceManager>()
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device
					.acquire_backbuffer()
					.expect("Swapchain failed to find a back buffer");

				let hello_world: &HelloWorld = Engine::module().unwrap();

				let mut painter = Painter::new();
				painter.fill_rect(&hello_world.style, (100.0, 100.0, 400.0, 400.0));
				let (vertex_buffer, index_buffer) = painter.finish().unwrap();

				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::BLACK))
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit();

				device.display(&[receipt]);
			})
	}
}

define_run_module!(HelloWorld, "Hello World");
