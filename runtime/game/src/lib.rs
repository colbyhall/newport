#![feature(trait_alias)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![feature(const_type_name)]
#![allow(arithmetic_overflow)]

use engine::{
	Builder,
	Engine,
	Event,
	Module,
};
#[cfg(not(feature = "editor"))]
use gpu::{
	Gpu,
	GraphicsPipeline,
	GraphicsRecorder,
	Layout,
};
use math::{
	Vector2,
};

use asset::AssetManager;
use graphics::Graphics;

pub mod components;
pub mod ecs;
#[cfg(feature = "editor")]
pub(crate) mod editor;
pub mod game;
mod input;
pub mod render;
pub mod systems;

use components::register_components;
use game::GameState;
use input::*;
use render::{
	DrawList,
	FrameContainer,
};

use sync::join;

pub struct Game3d {
	game_state: GameState,
	frames: FrameContainer,
	input_state: InputState,

	#[cfg(not(feature = "editor"))]
	present_pipeline: AssetRef<GraphicsPipeline>,
}

impl Module for Game3d {
	fn new() -> Self {
		Self {
			game_state: GameState::new(),
			frames: FrameContainer::new(),
			input_state: InputState::default(),

			#[cfg(not(feature = "editor"))]
			present_pipeline: AssetRef::new("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		let builder = register_components(builder)
			.module::<Graphics>()
			.module::<AssetManager>()
			.module::<ecs::Ecs>()
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
								.set_cursor_position(platform::winit::dpi::PhysicalPosition::new(
									size.width / 2,
									size.height / 2,
								))
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

				let Game3d {
					game_state, frames, ..
				} = game3d;

				Engine::wait_on(async {
					let simulation = async {
						game_state.simulate(dt).await;
						let scene = DrawList::build(game_state).await;
						frames.push_scene(scene);
					};
					let render = frames.render_scene();

					join!(simulation, render)
				});

				frames.advance_frame();

				// UNSAFE: Nothing should be touching input by this time
				let game3d: &mut Game3d = unsafe { Engine::module_mut().unwrap() };
				game3d.input_state.last_key_down = game3d.input_state.key_down;
				game3d.input_state.last_mouse_button_down = game3d.input_state.mouse_button_down;
				game3d.input_state.last_mouse_location = game3d.input_state.mouse_location.take();
				game3d.input_state.mouse_delta = Vector2::ZERO;
			});

		#[cfg(feature = "editor")]
		let builder = builder.module::<editor::Editor>();

		#[cfg(not(feature = "editor"))]
		let builder = builder.display(|| {
			let game3d: &Game3d = Engine::module().unwrap();

			let device = Gpu::device();
			let backbuffer = device
				.acquire_backbuffer()
				.expect("Swapchain failed to find a back buffer");

			let receipt = match game3d.frames.to_display() {
				Some(scene) => GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.bind_pipeline(&game3d.present_pipeline)
							.bind_texture("texture", &scene.diffuse_buffer)
							.draw(3, 0)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit(),
				None => GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::BLACK))
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit(),
			};

			device.display(&[receipt]);
		});

		#[allow(clippy::let_and_return)]
		builder
	}
}
