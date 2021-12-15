use {
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Buffer,
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
		Layout,
	},
	graphics::{
		FontCollection,
		Graphics,
		Painter,
	},
	math::{
		Color,
		Mat4,
		Vec2,
	},
	resources::{
		Handle,
		ResourceManager,
	},
};

struct Draw2d {
	pipeline: Handle<GraphicsPipeline>,
	font: Handle<FontCollection>,
}

impl Module for Draw2d {
	fn new() -> Self {
		Self {
			pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
			font: Handle::default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<ResourceManager>()
			.display(|| {
				let draw2d: &Draw2d = Engine::module().unwrap();
				let window = Engine::window().unwrap();
				let dpi = window.scale_factor() as f32;

				let font = draw2d.font.read();
				let font14 = font.font_at_size(14, dpi).unwrap();
				let font16 = font.font_at_size(16, dpi).unwrap();
				let font32 = font.font_at_size(32, dpi).unwrap();

				let (vertices, indices) = Painter::new()
					.fill_rect((100.0, 100.0, 200.0, 200.0), Color::GREEN)
					.text("Hello World!", Color::WHITE, (100.0, 300.0), &font14)
					.text("Hello World!", Color::WHITE, (100.0, 400.0), &font16)
					.text("Hello World!", Color::WHITE, (100.0, 500.0), &font32)
					.finish()
					.unwrap();

				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();
				let pipeline = draw2d.pipeline.read();

				let viewport = window.inner_size();
				let viewport = Vec2::new(viewport.width as f32 / dpi, viewport.height as f32 / dpi);

				let proj = Mat4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
				let view = Mat4::translate([-viewport.x / 2.0, -viewport.y / 2.0, 0.0]);

				#[allow(dead_code)]
				struct Imports {
					view: Mat4,
				}

				let imports =
					Buffer::new(gpu::BufferUsage::CONSTANTS, gpu::MemoryType::HostVisible, 1)
						.unwrap();
				imports.copy_to(&[Imports { view: proj * view }]).unwrap();

				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.bind_pipeline(&pipeline)
							.bind_vertex_buffer(&vertices)
							.bind_index_buffer(&indices)
							.bind_constants("imports", &imports, 0)
							.draw_indexed(indices.len(), 0)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.finish()
					.submit();

				device.display(&[receipt]);
				device.wait_for_idle();
			})
	}
}

define_run_module!(Draw2d, "Draw2d");
