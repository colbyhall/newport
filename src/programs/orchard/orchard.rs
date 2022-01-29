use {
	ecs::Ecs,
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Gpu,
		GraphicsRecorder,
		Layout,
	},
	graphics::Graphics,
	math::Color,
	resources::ResourceManager,
};

pub struct Game;
impl Module for Game {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Ecs>()
			.module::<Graphics>()
			.module::<ResourceManager>()
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();
				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::RED))
					.texture_barrier(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.submit();
				device.display(&[receipt]);
			})
	}
}

define_run_module!(Game, "Orchard");
