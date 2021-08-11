
use engine::{
	Builder,
	Engine,
	Module,
};

use math::Color;

use gpu::Gpu;
use graphics::Graphics;

use asset::AssetManager;

pub struct Game;

impl Module for Game {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			// .tick(|_engine: &Engine, dt: f32| {
			// 	// let game = engine.module::<Game>().unwrap();

			// 	// Simulate the game state and then build the render state
			// 	// let new_render_state = {
			// 	// 	let mut game_state = game.game_state.lock().unwrap();
			// 	// 	game_state.simulate(dt)
			// 	// };

			// 	// {
			// 	// 	let mut render_state = game.render_state.write().unwrap();
			// 	// 	*render_state = Some(new_render_state);
			// 	// }
			// })
			.display(|engine: &Engine| {
				let gpu: &Gpu = engine.module().unwrap();
				let device = gpu.device();

				let backbuffer = device.acquire_backbuffer();
				let render_pass = gpu.backbuffer_render_pass();

				let gfx = device
					.create_graphics_recorder()
					.render_pass(&render_pass, &[&backbuffer], |ctx| ctx.clear(Color::GREEN))
					.resource_barrier_texture(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.finish();

				let receipt = device.submit_graphics(vec![gfx], &[]);
				device.display(&[receipt]);
				device.wait_for_idle();
			})
			.module::<Graphics>()
			.module::<AssetManager>()
	}
}
