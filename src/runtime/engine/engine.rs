#![feature(trait_alias)]
#![feature(string_remove_matches)]

mod builder;
mod config;
mod log;
mod module;
mod uuid;

#[cfg(test)]
mod test;

pub use {
	builder::*,
	config::*,
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
	thread::ThreadId,
	time::Instant,
};

use os::{
	winit::event::{
		DeviceEvent,
		ElementState,
		MouseButton,
		MouseScrollDelta,
	},
	ControlFlow,
	Event as WinEvent,
	EventLoop,
	Input,
	Window,
	WindowBuilder,
	WindowEvent,
};

pub use os::input;

static mut ENGINE: Option<Engine> = None;

pub const ENGINE_NAME: &str = "Newport";
pub const ENGINE_VERSION: &str = "0.1";

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

	main: ThreadId,
}

impl Engine {
	pub(crate) fn spawn(
		mut builder: Builder,
		window: Option<Window>,
		test: bool,
	) -> Result<(), std::io::Error> {
		unsafe {
			// Use this to mark when registration finished. This must happen before anything else.
			let registration_finish_time = Instant::now()
				.duration_since(builder.creation)
				.as_secs_f64() * 1000.0;

			// Ensure that we're working in the projects workspace.
			let exe_path = std::env::current_exe()?;
			let new_working_directory = if test {
				exe_path
					.parent()
					.unwrap()
					.parent()
					.unwrap()
					.parent()
					.unwrap()
					.parent()
					.unwrap()
			} else {
				exe_path
					.parent()
					.unwrap()
					.parent()
					.unwrap()
					.parent()
					.unwrap()
			};
			std::env::set_current_dir(new_working_directory)?;

			// Ensure we have a valid name for the project. This is used for a variety of things
			let name = builder.name.take().unwrap_or_else(|| "project".to_string());

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

				main: std::thread::current().id(),
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
				engine.modules.insert(it.id, (it.spawn)());
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
			None => "project".to_string(),
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

		Engine::spawn(builder, window, false)?;
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
				WinEvent::WindowEvent {
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
				WinEvent::WindowEvent {
					event: WindowEvent::KeyboardInput { input, .. },
					..
				} => {
					if let Some(virtual_keycode) = input.virtual_keycode {
						let key = os::virtual_keycode_to_input(virtual_keycode);
						let event = Event::Key {
							key,
							pressed: input.state == ElementState::Pressed,
						};

						process_input.iter().for_each(|process| process(&event));
					}
				}
				WinEvent::DeviceEvent {
					event: DeviceEvent::MouseMotion { delta },
					..
				} => {
					let event = Event::MouseMotion(delta.0 as f32, delta.1 as f32);

					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::WindowEvent {
					event: WindowEvent::MouseInput { button, state, .. },
					..
				} => {
					let mouse_button = match button {
						MouseButton::Left => os::MOUSE_BUTTON_LEFT,
						MouseButton::Right => os::MOUSE_BUTTON_RIGHT,
						MouseButton::Middle => os::MOUSE_BUTTON_MIDDLE,
						_ => unimplemented!(),
					};
					let event = Event::MouseButton {
						mouse_button,
						pressed: state == ElementState::Pressed,
					};

					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::WindowEvent {
					event: WindowEvent::CursorMoved { position, .. },
					..
				} => {
					let event = Event::MouseMove(position.x as f32, height - position.y as f32);
					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					let event = Event::Resized(size.width, size.height);
					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::WindowEvent {
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
				WinEvent::WindowEvent {
					event: WindowEvent::ReceivedCharacter(c),
					..
				} => {
					let event = Event::Char(c);
					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::WindowEvent {
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
				WinEvent::WindowEvent {
					event: WindowEvent::CursorEntered { .. },
					..
				} => {
					let event = Event::MouseEnter;
					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::WindowEvent {
					event: WindowEvent::CursorLeft { .. },
					..
				} => {
					let event = Event::MouseLeave;
					process_input.iter().for_each(|process| process(&event));
				}
				WinEvent::MainEventsCleared => {
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
				WinEvent::RedrawRequested(_) => match &display {
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

	/// Returns an immutable reference
	pub fn module<'a, T: Module>() -> Option<&'a T> {
		let engine = Engine::as_ref();

		let id = TypeId::of::<T>();

		let module = engine.modules.get(&id)?;
		module.downcast_ref::<T>()
	}

	/// Returns a mutable reference to a `Module`
	///
	/// # Safety
	///
	/// Modules can be accessed on any thread. This does not provide a locking mechanism.
	pub unsafe fn module_mut<'a, T: Module>() -> Option<&'a mut T> {
		let engine = Engine::as_mut();

		let id = TypeId::of::<T>();
		let module = engine.modules.get_mut(&id)?;
		module.downcast_mut::<T>()
	}

	/// Returns a mutable reference to a `Module` if it is marked as LOCAL and is on main thread
	pub fn module_mut_checked<'a, T: Module>() -> Option<&'a mut T> {
		let current = std::thread::current().id();
		let engine = unsafe { Engine::as_mut() };
		assert_eq!(
			current, engine.main,
			"Engine::module_mut_checked must be called with a LOCAL module on the main thread"
		);
		assert!(T::LOCAL);

		let id = TypeId::of::<T>();
		let module = engine.modules.get_mut(&id)?;
		module.downcast_mut::<T>()
	}

	pub fn register<'a, T: Register>() -> &'a [T] {
		let engine = Engine::as_ref();
		let id = TypeId::of::<T>();

		match engine.registers.get(&id) {
			Some(reg) => reg.downcast_ref::<Vec<T>>().unwrap(),
			None => &[],
		}
	}

	pub fn config<'a, T: Config>() -> &'a T {
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
	pub fn name<'a>() -> &'a str {
		&Engine::as_ref().name
	}

	/// Returns the window that the engine draws into
	pub fn window<'a>() -> Option<&'a Window> {
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

	pub fn logger<'a>() -> &'a Logger {
		&Engine::as_ref().logger
	}
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
	FocusGained,
	FocusLost,
	Key { key: Input, pressed: bool },
	Resized(u32, u32),
	Char(char),
	MouseWheel(f32, f32),
	MouseButton { mouse_button: Input, pressed: bool },
	MouseMove(f32, f32),
	MouseLeave,
	MouseEnter,
	MouseMotion(f32, f32),
}
