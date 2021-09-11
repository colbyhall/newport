use newport::{
	asset::AssetManager,
	engine::{
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Gpu,
		GraphicsRecorder,
		Layout,
	},
	math,
};

// First thing first is to define our module struct
struct GpuExample;

// Implement the module trait
impl Module for GpuExample {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.display(|| {
				let device = Gpu::device();

				let backbuffer = device
					.acquire_backbuffer()
					.expect("Something is wrong with the swapchain");

				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| ctx.clear_color(math::Color::GREEN))
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit();
				device.display(&[receipt]);
			})
			.module::<AssetManager>()
			.module::<Gpu>()
	}
}

// Start the app runner
fn main() -> Result<(), std::io::Error> {
	Engine::builder()
		.module::<GpuExample>()
		.name("Gpu Example")
		.run()
}
