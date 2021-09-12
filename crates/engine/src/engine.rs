use crate::{
	Builder,
	Event,
	Module,
	Register,
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

use platform::winit::event::DeviceEvent;
use platform::winit::{
	event::{
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

use sync::executor::ThreadPool;
use sync::Future;

static mut ENGINE: Option<Engine> = None;

/// Global runnable structure used for instantiating engine modules and handling app code
///
/// Created using an [`Builder`] which defines the functionality of the app using [`Module`]s
pub struct Engine {
	name: String,
	modules: HashMap<TypeId, Box<dyn Any>>,
	registers: HashMap<TypeId, Box<dyn Any>>,

	is_running: AtomicBool,
	fps: AtomicI32,

	executor: Option<ThreadPool>, // Not available during module initialization process

	window: Option<Window>,
}

impl Engine {
	pub(crate) fn run(mut builder: Builder) -> Result<(), std::io::Error> {
		let event_loop = EventLoop::new();

		// UNSAFE: Set the global state
		let engine = unsafe {
			// let id = TypeId::of::<WindowStyle>();
			// let styles: Vec<WindowStyle> = match builder.registers.get(&id) {
			// Some(any_vec) => any_vec.downcast_ref::<Vec<WindowStyle>>().unwrap().clone(),
			// None => Vec::default(),
			// };
			// let style = match styles.last() {
			// Some(style) => *style,
			// None => WindowStyle::Windowed,
			// };
			let name = builder.name.take().unwrap_or_else(|| "newport".to_string());

			let window = match &builder.display {
				Some(_) => WindowBuilder::new()
					.with_title(name.clone())
					.with_maximized(true)
					.with_visible(false)
					.build(&event_loop)
					.ok(),
				None => None,
			};

			ENGINE = Some(Engine {
				name,
				modules: HashMap::with_capacity(builder.entries.len()),
				registers: builder.registers.take().unwrap(),

				is_running: AtomicBool::new(true),
				fps: AtomicI32::new(0),

				executor: None,

				window,
			});

			let engine = ENGINE.as_mut().unwrap();

			// NOTE: All modules a module depends on will be available at initialization
			builder.entries.drain(..).for_each(|it| {
				engine.modules.insert(it.id, (it.spawn)());
			});

			// Initializer after module initialization because engine is being modified on main thread during module initialization
			engine.executor = Some(ThreadPool::new()?);

			ENGINE.as_ref().unwrap()
		};

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

						builder
							.process_input
							.iter()
							.for_each(|process| process(&event));
					}
				}
				WinitEvent::DeviceEvent {
					event: DeviceEvent::MouseMotion { delta },
					..
				} => {
					let event = Event::MouseMotion(delta.0 as f32, delta.1 as f32);

					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
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

					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorMoved { position, .. },
					..
				} => {
					let event = Event::MouseMove(position.x as f32, height - position.y as f32);
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					let event = Event::Resized(size.width, size.height);
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
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
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::ReceivedCharacter(c),
					..
				} => {
					let event = Event::Char(c);
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
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
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorEntered { .. },
					..
				} => {
					let event = Event::MouseEnter;
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorLeft { .. },
					..
				} => {
					let event = Event::MouseLeave;
					builder
						.process_input
						.iter()
						.for_each(|process| process(&event));
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

					builder.tick.iter().for_each(|tick| tick(dt));

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
				WinitEvent::RedrawRequested(_) => match &builder.display {
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

	pub fn register<T: Register>() -> Vec<T> {
		let engine = Engine::as_ref();
		let id = TypeId::of::<T>();

		match engine.registers.get(&id) {
			Some(reg) => reg.downcast_ref::<Vec<T>>().unwrap().clone(),
			None => Default::default(),
		}
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
		Engine::as_ref()
			.executor
			.as_ref()
			.expect("ThreadPool executor is only avaible after module initialization.");

		sync::block_on(future)
	}
}
