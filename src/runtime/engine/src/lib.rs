#![feature(trait_alias)]
#![feature(string_remove_matches)]

mod builder;
mod config;
mod event;
mod log;
mod module;
mod uuid;

#[cfg(test)]
mod test;

pub use {
	builder::*,
	config::*,
	event::*,
	log::*,
	module::*,
	uuid::*,
};

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
	sync::atomic::{
		AtomicBool,
		AtomicI32,
		Ordering,
	},
	time::Instant,
};

use platform::winit::{
	event::{
		DeviceEvent,
		ElementState,
		Event as WinitEvent,
		MouseButton,
		MouseScrollDelta,
		WindowEvent,
	},
	event_loop::{
		ControlFlow,
		EventLoop,
	},
	window::{
		Window,
		WindowBuilder,
	},
};

use platform::input::Input;

use sync::Future;

static mut ENGINE: Option<Engine> = None;

/// Global runnable structure used for instantiating engine modules and handling app code
///
/// Created using a [`Builder`] which defines the functionality of the app using [`Module`]s
pub struct Engine {
	name: String,
	modules: HashMap<TypeId, Box<dyn Any>>,
	registers: HashMap<TypeId, Box<dyn Any>>,

	is_running: AtomicBool,
	fps: AtomicI32,

	window: Option<Window>,

	logger: Logger,
	config: ConfigMap,
}

impl Engine {
	pub(crate) fn spawn(
		mut builder: Builder,
		window: Option<Window>,
	) -> Result<(), std::io::Error> {
		unsafe {
			// Use this to mark when registration finished. This must happen before anything else.
			let registration_finish_time = Instant::now()
				.duration_since(builder.creation)
				.as_secs_f64() * 1000.0;

			// Ensure that we're working in the projects workspace.
			let exe_path = std::env::current_exe()?;
			#[cfg(not(test))]
			let new_working_directory = exe_path
				.parent()
				.unwrap()
				.parent()
				.unwrap()
				.parent()
				.unwrap();
			#[cfg(test)]
			let new_working_directory = exe_path
				.parent()
				.unwrap()
				.parent()
				.unwrap()
				.parent()
				.unwrap()
				.parent()
				.unwrap();
			println!("{:?}", new_working_directory);
			std::env::set_current_dir(new_working_directory)?;

			// Ensure we have a valid name for the project. This is used for a variety of things
			let name = builder.name.take().unwrap_or_else(|| "newport".to_string());

			// Manually clone the config register as the config map needs it. Config must happen before module initialization so they can rely on it
			let id = TypeId::of::<ConfigRegister>();
			let config_registers: Vec<ConfigRegister> =
				match builder.registers.as_ref().unwrap().get(&id) {
					Some(any_vec) => any_vec
						.downcast_ref::<Vec<ConfigRegister>>()
						.unwrap()
						.clone(),
					None => Vec::default(),
				};

			ENGINE = Some(Engine {
				name,
				modules: HashMap::with_capacity(builder.modules.len()),
				registers: builder.registers.take().unwrap(),

				is_running: AtomicBool::new(true),
				fps: AtomicI32::new(0),

				window,

				logger: Logger::new(),
				config: ConfigMap::new(config_registers),
			});

			info!(
				ENGINE_CATEGORY,
				"Registration process took {:.2}ms", registration_finish_time
			);

			let engine = ENGINE.as_mut().unwrap();

			info!(ENGINE_CATEGORY, "Starting module initialization.");
			let now = Instant::now();

			// NOTE: All modules a module depends on will be available at initialization
			builder.modules.drain(..).for_each(|it| {
				info!(ENGINE_CATEGORY, "Started initializing module {}", it.name);
				let now = Instant::now();
				engine.modules.insert(it.id, (it.spawn)());
				let dur = Instant::now().duration_since(now).as_secs_f64() * 1000.0;
				info!(
					ENGINE_CATEGORY,
					"Finished initializing module {}. ({:.2}ms)", it.name, dur
				);
			});

			let dur = Instant::now().duration_since(now).as_secs_f64() * 1000.0;
			info!(ENGINE_CATEGORY, "Module initialization took {:.2}ms.", dur);

			Ok(())
		}
	}

	pub(crate) fn run(mut builder: Builder) -> Result<(), std::io::Error> {
		let display = builder.display.take();
		let process_input = std::mem::take(&mut builder.process_input);
		let tick = std::mem::take(&mut builder.tick);

		let name = match &builder.name {
			Some(name) => name.clone(),
			None => "newport".to_string(),
		};

		let event_loop = EventLoop::new();
		let window = match &display {
			Some(_) => WindowBuilder::new()
				.with_title(name)
				.with_maximized(true)
				.with_visible(false)
				.build(&event_loop)
				.ok(),
			None => None,
		};

		Engine::spawn(builder, window)?;
		let engine = Engine::as_ref();

		let mut frame_count = 0;
		let mut time = 0.0;
		let mut displayed = false;
		let mut do_first_show = true;

		let mut last_frame_time = Instant::now();
		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Poll;

			let height = if let Some(window) = Engine::window() {
				window.inner_size().height as f32
			} else {
				0.0
			};

			match event {
				WinitEvent::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => {
					*control_flow = ControlFlow::Exit;

					// Set the window to be invisible immedietely
					if let Some(window) = Engine::window() {
						window.set_maximized(false);
						window.set_visible(false);
					}
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::KeyboardInput { input, .. },
					..
				} => {
					if let Some(virtual_keycode) = input.virtual_keycode {
						let key = Input::key_from_code(virtual_keycode)
							.unwrap_or(platform::input::UNKNOWN);
						let event = Event::Key {
							key,
							pressed: input.state == ElementState::Pressed,
						};

						process_input.iter().for_each(|process| process(&event));
					}
				}
				WinitEvent::DeviceEvent {
					event: DeviceEvent::MouseMotion { delta },
					..
				} => {
					let event = Event::MouseMotion(delta.0 as f32, delta.1 as f32);

					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::MouseInput { button, state, .. },
					..
				} => {
					let mouse_button = match button {
						MouseButton::Left => platform::input::MOUSE_BUTTON_LEFT,
						MouseButton::Right => platform::input::MOUSE_BUTTON_RIGHT,
						MouseButton::Middle => platform::input::MOUSE_BUTTON_MIDDLE,
						_ => unimplemented!(),
					};
					let event = Event::MouseButton {
						mouse_button,
						pressed: state == ElementState::Pressed,
					};

					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorMoved { position, .. },
					..
				} => {
					let event = Event::MouseMove(position.x as f32, height - position.y as f32);
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					let event = Event::Resized(size.width, size.height);
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::MouseWheel { delta, .. },
					..
				} => {
					let event = match delta {
						MouseScrollDelta::LineDelta(x, y) => Event::MouseWheel(x as f32, y as f32),
						MouseScrollDelta::PixelDelta(dif) => {
							Event::MouseWheel(dif.x as f32, dif.y as f32)
						}
					};
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::ReceivedCharacter(c),
					..
				} => {
					let event = Event::Char(c);
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::Focused(focused),
					..
				} => {
					let event = if focused {
						Event::FocusGained
					} else {
						Event::FocusLost
					};
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorEntered { .. },
					..
				} => {
					let event = Event::MouseEnter;
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorLeft { .. },
					..
				} => {
					let event = Event::MouseLeave;
					process_input.iter().for_each(|process| process(&event));
				}
				WinitEvent::MainEventsCleared => {
					let now = Instant::now();
					let dt = now.duration_since(last_frame_time).as_secs_f32();
					last_frame_time = now;

					time += dt;
					if time >= 1.0 {
						time = 0.0;
						engine.fps.store(frame_count, Ordering::Relaxed);
						frame_count = 0;
					}
					frame_count += 1;

					tick.iter().for_each(|tick| tick(dt));

					if !displayed {
						if let Some(window) = engine.window.as_ref() {
							window.request_redraw();
						}
					} else {
						displayed = false;
					}

					if do_first_show {
						if let Some(window) = engine.window.as_ref() {
							window.set_visible(true);
						}
						do_first_show = false;
					}
				}
				WinitEvent::RedrawRequested(_) => match &display {
					Some(display) => {
						(display)();
						displayed = true;

						if do_first_show {
							if let Some(window) = engine.window.as_ref() {
								window.set_visible(true);
							}
							do_first_show = false;
						}
					}
					None => {}
				},
				_ => (),
			}
		});
	}

	/// Returns the global [`Engine`] as a ref
	fn as_ref() -> &'static Engine {
		unsafe { ENGINE.as_ref().unwrap() }
	}

	unsafe fn as_mut() -> &'static mut Engine {
		ENGINE.as_mut().unwrap()
	}

	pub fn module<T: Module>() -> Option<&'static T> {
		let engine = Engine::as_ref();

		let id = TypeId::of::<T>();

		let module = engine.modules.get(&id)?;
		module.downcast_ref::<T>()
	}

	pub unsafe fn module_mut<T: Module>() -> Option<&'static mut T> {
		let engine = Engine::as_mut();

		let id = TypeId::of::<T>();
		let module = engine.modules.get_mut(&id)?;
		module.downcast_mut::<T>()
	}

	pub fn register<T: Register>() -> Option<&'static [T]> {
		let engine = Engine::as_ref();
		let id = TypeId::of::<T>();

		match engine.registers.get(&id) {
			Some(reg) => Some(reg.downcast_ref::<Vec<T>>().unwrap()),
			None => None,
		}
	}

	pub fn config<T: Config>() -> &'static T {
		let engine = Engine::as_ref();
		let id = TypeId::of::<T>();

		let entry = engine.config.entries.get(&id).unwrap_or_else(|| {
			panic!(
				"Config of type \"{}\" is not registered.",
				std::any::type_name::<T>()
			)
		});

		entry.value.downcast_ref().unwrap()
	}

	/// Returns the name of the engine runnable
	pub fn name() -> &'static str {
		&Engine::as_ref().name
	}

	/// Returns the window that the engine draws into
	pub fn window() -> Option<&'static Window> {
		Engine::as_ref().window.as_ref()
	}

	pub fn shutdown() {
		Engine::as_ref().is_running.store(false, Ordering::Relaxed);
	}

	pub fn fps() -> i32 {
		Engine::as_ref().fps.load(Ordering::Relaxed)
	}

	pub fn builder() -> Builder {
		Builder::new()
	}

	pub fn wait_on<F: Future + Send>(future: F) -> F::Output {
		sync::block_on(future)
	}

	pub fn logger() -> &'static Logger {
		&Engine::as_ref().logger
	}
}
