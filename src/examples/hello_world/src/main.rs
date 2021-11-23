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

	time: f32,

	draw_pipeline: Handle<GraphicsPipeline>,
	texture: Handle<Texture>,
}

impl Module for HelloWorld {
	fn new() -> Self {
		let mut style = PainterStyle::default();
		style.line_width = 20.0;
		Self {
			style,
			time: 0.0,
			draw_pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
			texture: Handle::find_or_load("{44f1bd76-ea26-424b-8871-59de27a1413a}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<ResourceManager>()
			.tick(|dt| {
				let hello_world: &mut HelloWorld = unsafe { Engine::module_mut().unwrap() };
				hello_world.time += dt * 1.0;
			})
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device
					.acquire_backbuffer()
					.expect("Swapchain failed to find a back buffer");

				let hello_world: &HelloWorld = Engine::module().unwrap();
				let style = &hello_world.style;

				let mut painter = Painter::new();
				painter.stroke_rect(style, (100.0, 100.0, 500.0, 500.0));
				painter.fill_rect(style, (520.0, 100.0, 920.0, 500.0));
				painter.textured_rect(
					style,
					(940.0, 100.0, 1340.0, 500.0),
					&hello_world.texture,
					(0.0, 0.0, 1.0, 1.0),
				);

				painter.stroke_rect(style, (100.0, 520.0, 920.0, 920.0));

				let mut wave = |offset: f32| {
					let xy = vec2!(110.0, 720.0);
					let amplitude = 150.0 - style.line_width * 2.0;
					for index in 0..800 {
						let t = index as f32;

						let ax = t - 0.1;
						let bx = t + 1.1;

						let ay = ax / 100.0 + hello_world.time;
						let by = bx / 100.0 + hello_world.time;

						let func = |x: f32| x.sin();

						let a = xy + vec2!(ax, func(ay + offset) * amplitude);
						let b = xy + vec2!(bx, func(by + offset) * amplitude);
						painter.stroke(style, a, b);
					}
				};

				wave(0.0);
				wave(PI / 4.0);
				wave(PI / 2.0);
				wave((PI / 4.0) * 3.0);
				wave(PI);

				let (vertex_buffer, index_buffer) = painter.finish().unwrap();

				#[allow(dead_code)]
				struct Imports {
					view: Matrix4,
				}

				let scale = Engine::window().unwrap().scale_factor() as f32;
				let viewport = vec2!(backbuffer.width() as f32, backbuffer.height() as f32) / scale;

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
