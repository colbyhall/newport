use newport::{
	asset,
	engine::{
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Gpu,
		Layout,
	},
	math,
};

use asset::AssetManager;

// First thing first is to define our module struct
struct GpuExample;

// Implement the module trait
impl Module for GpuExample {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.tick(|engine: &Engine, _dt| {
				let gpu = engine.module::<Gpu>().unwrap();
				let device = gpu.device();

				let backbuffer = device.acquire_backbuffer();
				let render_pass = gpu.backbuffer_render_pass();

				let gfx = device
					.create_graphics_recorder()
					.render_pass(&render_pass, &[&backbuffer], |recorder| {
						recorder.clear(math::Color::GREEN)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.finish();

				let receipt = device.submit_graphics(vec![gfx], &[]);
				device.display(&[receipt]);
				device.wait_for_idle();
			})
			.module::<AssetManager>()
			.module::<Gpu>()
	}
}

// Start the app runner
fn main() {
	Builder::new()
		.module::<GpuExample>()
		.name("Gpu Example")
		.run()
}
