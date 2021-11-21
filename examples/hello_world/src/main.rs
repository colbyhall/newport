use {
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::*,
	graphics::*,

	math::*,

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

				#[allow(dead_code)]
				struct Imports {
					view: Matrix4,
				}

				let viewport = Vector2::new(backbuffer.width() as f32, backbuffer.height() as f32);

				let proj = Matrix4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
				let view =
					Matrix4::translate(Vector3::new(-viewport.x / 2.0, -viewport.y / 2.0, 0.0));

				let imports =
					Buffer::new(BufferUsage::CONSTANTS, gpu::MemoryType::HostVisible, 1).unwrap();
				imports.copy_to(&[Imports { view: proj * view }]).unwrap();

				let pipeline = hello_world.draw_pipeline.read();

				let receipt = GraphicsRecorder::new()
					.resource_barrier_texture(
						&backbuffer,
						Layout::Undefined,
						Layout::ColorAttachment,
					)
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.bind_pipeline(&pipeline)
							.bind_vertex_buffer(&vertex_buffer)
							.bind_index_buffer(&index_buffer)
							.bind_constants("imports", &imports, 0)
							.draw_indexed(index_buffer.len(), 0)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit();

				device.display(&[receipt]);
			})
	}
}

define_run_module!(HelloWorld, "Hello World");
