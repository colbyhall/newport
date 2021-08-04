use crate::{
	GameState,
	RenderState,
};

use engine::{
	Builder,
	Engine,
	Module,
};

use math::Color;

use gpu::Gpu;
use graphics::Graphics;

use asset::AssetManager;

use std::sync::{
	Mutex,
	RwLock,
};

pub struct Game {
	pub game_state: Mutex<GameState>,
	pub render_state: RwLock<Option<RenderState>>,
}

impl Module for Game {
	fn new() -> Self {
		Self {
			game_state: Mutex::new(GameState::new()),
			render_state: RwLock::new(None),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.tick(|engine: &Engine, dt: f32| {
				let game = engine.module::<Game>().unwrap();

				// Simulate the game state and then build the render state
				let new_render_state = {
					let mut game_state = game.game_state.lock().unwrap();
					game_state.simulate(dt)
				};

				{
					let mut render_state = game.render_state.write().unwrap();
					*render_state = Some(new_render_state);
				}
			})
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
