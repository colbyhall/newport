mod layout;
mod widget;

pub use {
	layout::*,
	widget::*,
};

use {
	engine::{
		Engine,
		Event,
		Module,
	},
	gpu::{
		Buffer,
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
	},
	graphics::{
		Graphics,
		Painter,
	},
	math::*,
	resources::{
		Handle,
		ResourceManager,
	},
	std::thread,
};

pub struct Gui {
	thread_id: thread::ThreadId,

	widgets: WidgetTree,
	layouts: LayoutTree,

	pipeline: Handle<GraphicsPipeline>,
}

impl Gui {
	fn as_ref<'a>() -> &'a Gui {
		let gui: &'a Gui = Engine::module().unwrap();
		assert_eq!(thread::current().id(), gui.thread_id);
		gui
	}

	fn as_mut<'a>() -> &'a mut Gui {
		let gui: &'a mut Gui = unsafe { Engine::module_mut().unwrap() };
		assert_eq!(thread::current().id(), gui.thread_id);
		gui
	}

	pub fn widgets<'a>() -> &'a WidgetTree {
		&Self::as_ref().widgets
	}

	pub fn widgets_mut<'a>() -> &'a mut WidgetTree {
		&mut Self::as_mut().widgets
	}

	pub fn layouts<'a>() -> &'a LayoutTree {
		&Self::as_ref().layouts
	}

	pub fn layouts_mut<'a>() -> &'a mut LayoutTree {
		&mut Self::as_mut().layouts
	}
}

impl Module for Gui {
	fn new() -> Self {
		Self {
			thread_id: thread::current().id(),

			widgets: WidgetTree::new(),
			layouts: LayoutTree::new(),

			pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: engine::Builder) -> engine::Builder {
		builder
			.module::<Graphics>()
			.module::<ResourceManager>()
			.process_input(|event| match event {
				Event::Resized(_, _) => Gui::layouts_mut().rebuild(),
				_ => {}
			})
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();

				let receipt = if let Some(base) = Gui::widgets().base() {
					let mut painter = Painter::new();

					if let Some(widget) = Gui::widgets().find(base) {
						let layout = Gui::layouts().find(base);
						let bounds = Rect::from_min_max(layout.absolute_position, layout.size);

						widget.paint(&mut painter, bounds);
					}
					let (vertex_buffer, index_buffer) = painter.finish().unwrap();

					// Idomatic way of garbbing the viewport size
					let viewport = engine::Engine::window().unwrap().inner_size();
					let viewport = Vec2::new(viewport.width as f32, viewport.height as f32);

					let projection = Matrix4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
					let view = Matrix4::translate([-viewport.x / 2.0, -viewport.y / 2.0, 0.0]);

					let constants =
						Buffer::new(gpu::BufferUsage::CONSTANTS, gpu::MemoryType::HostVisible, 1)
							.unwrap();
					constants.copy_to(&[projection * view]).unwrap();

					let pipeline = Gui::as_ref().pipeline.read();
					GraphicsRecorder::new()
						.render_pass(&[&backbuffer], |ctx| {
							ctx.clear_color(Color::BLACK)
								.bind_pipeline(&pipeline)
								.bind_vertex_buffer(&vertex_buffer)
								.bind_index_buffer(&index_buffer)
								.bind_constants("imports", &constants, 0)
								.draw_indexed(index_buffer.len(), 0)
						})
						.resource_barrier_texture(
							&backbuffer,
							gpu::Layout::ColorAttachment,
							gpu::Layout::Present,
						)
						.finish()
						.submit()
				} else {
					GraphicsRecorder::new()
						.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::MAGENTA))
						.resource_barrier_texture(
							&backbuffer,
							gpu::Layout::ColorAttachment,
							gpu::Layout::Present,
						)
						.finish()
						.submit()
				};

				device.display(&[receipt]);
				device.wait_for_idle();
			})
	}
}
