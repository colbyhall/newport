use asset::AssetRef;
use engine::{
	Builder,
	Engine,
	Module,
	Event,
};
use math::{ Vector2, Color };

use gpu::GraphicsPipeline;
use gpu::{ GraphicsRecorder, Gpu, Layout };

use asset::AssetManager;
use graphics::Graphics;

pub mod components;
use components::register_components;

pub mod game;
use game::GameState;

pub mod render;
use render::{ FrameContainer, Scene };

mod input;
use input::*;

pub mod systems;

use sync::join;

pub struct Game3d {
	game_state: GameState,
	frames: FrameContainer,
	input_state: InputState,

	present_pipeline: AssetRef<GraphicsPipeline>,
}

impl Module for Game3d {
	fn new() -> Self {
		Self {
			game_state: GameState::new(),
			frames: FrameContainer::new(),
			input_state: InputState::default(),

			present_pipeline: AssetRef::new("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}").unwrap_or_default()
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		register_components(builder)
			.module::<Graphics>()
			.module::<AssetManager>()
			.process_input(|event| {
				let game3d: &mut Game3d = unsafe { Engine::module_mut().unwrap() };

				let window = Engine::window().unwrap();

				match event {
					Event::Key { key, pressed } => {
						game3d.input_state.key_down[key.as_key().0 as usize] = *pressed
					}
					Event::MouseMove(x, y) => {
						if game3d.input_state.mouse_locked {
							let size = window.inner_size();
							if *x as u32 == size.width / 2 && *y as u32 == size.height / 2 {
								return;
							}

							window
								.set_cursor_position(
									platform::winit::dpi::PhysicalPosition::new(
										size.width / 2,
										size.height / 2,
									),
								)
								.unwrap();
						}

						game3d.input_state.mouse_location = Some(Vector2::new(*x, *y));
					}
					Event::MouseMotion(x, y) => {
						game3d.input_state.mouse_delta = Vector2::new(*x, *y);
					}
					Event::MouseButton {
						mouse_button,
						pressed,
					} => {
						game3d.input_state.mouse_button_down
							[mouse_button.as_mouse_button() as usize] = *pressed
					}
					_ => {}
				}
			})
			.tick(|dt| {
				let game3d: &Game3d = Engine::module().unwrap();

				let Game3d { game_state, frames, .. } = game3d;

				Engine::wait_on(async {
					let simulation = async {
						game_state.simulate(dt).await;
						let scene = Scene::build(game_state).await;
						frames.push_scene(scene);
					};
					let render = frames.render_scene();
	
					join!(simulation, render)
				});

				frames.advance_frame();

				// UNSAFE: Nothing should be touching input by this time
				let game3d: &mut Game3d = unsafe{ Engine::module_mut().unwrap() };
				game3d.input_state.last_key_down = game3d.input_state.key_down;
				game3d.input_state.last_mouse_button_down = game3d.input_state.mouse_button_down;
				game3d.input_state.last_mouse_location = game3d.input_state.mouse_location.take();
				game3d.input_state.mouse_delta = Vector2::ZERO;
			})
			.display(|| {
				let game3d: &Game3d = Engine::module().unwrap();

				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().expect("Swapchain failed to find a back buffer");

				let receipt = match game3d.frames.to_display() {
					Some(scene) => {
						GraphicsRecorder::new()
							.render_pass(&[&backbuffer], |ctx| {
								ctx.bind_pipeline(&game3d.present_pipeline)
									.bind_texture(
										"texture",
										&scene.diffuse_buffer,
									)
									.draw(3, 0)
							})
							.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
							.submit()
					},
					None => {
						GraphicsRecorder::new()
							.render_pass(&[&backbuffer], |ctx| {
								ctx.clear_color(Color::BLACK)
							})
							.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
							.submit()
					}
				};

				device.display(&[receipt]);
			})
	}
}
