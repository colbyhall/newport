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

	window: Option<Window>,
}

impl Engine {
	pub(crate) fn run(mut builder: Builder) {
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
			let name = builder.name.take().unwrap_or("newport".to_string());

			let window = match &builder.display {
				Some(_) => WindowBuilder::new()
					.with_title(name.clone())
					.with_maximized(true)
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

				window,
			});

			let engine = ENGINE.as_mut().unwrap();

			// NOTE: All modules a module depends on will be available at initialization
			// TODO: Worry about safety here.
			builder.entries.drain(..).for_each(|it| {
				engine.modules.insert(it.id, (it.spawn)());
			});

			ENGINE.as_ref().unwrap()
		};

		// Do post init
		builder.post_inits.drain(..).for_each(|init| init(engine));

		match &engine.window {
			Some(window) => {
				window.set_visible(true);
			}
			None => {}
		}

		let mut frame_count = 0;
		let mut time = 0.0;

		let mut last_frame_time = Instant::now();
		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Poll;

			let height = engine.window().unwrap().inner_size().height as f32;

			match event {
				WinitEvent::WindowEvent {
					event: WindowEvent::CloseRequested,
					..
				} => {
					*control_flow = ControlFlow::Exit;
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::KeyboardInput { input, .. },
					..
				} => {
					let key = Input::key_from_code(input.scancode as u8)
						.unwrap_or(platform::input::UNKNOWN);
					let event = Event::Key {
						key,
						pressed: input.state == ElementState::Pressed,
					};

					builder
						.process_input
						.iter()
						.for_each(|process| process(engine, &event));
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
						.for_each(|process| process(engine, &event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorMoved { position, .. },
					..
				} => {
					let event = Event::MouseMove(position.x as f32, height - position.y as f32);
					builder
						.process_input
						.iter()
						.for_each(|process| process(engine, &event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::Resized(size),
					..
				} => {
					let event = Event::Resized(size.width, size.height);
					builder
						.process_input
						.iter()
						.for_each(|process| process(engine, &event));
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
						.for_each(|process| process(engine, &event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::ReceivedCharacter(c),
					..
				} => {
					let event = Event::Char(c);
					builder
						.process_input
						.iter()
						.for_each(|process| process(engine, &event));
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
						.for_each(|process| process(engine, &event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorEntered { .. },
					..
				} => {
					let event = Event::MouseEnter;
					builder
						.process_input
						.iter()
						.for_each(|process| process(engine, &event));
				}
				WinitEvent::WindowEvent {
					event: WindowEvent::CursorLeft { .. },
					..
				} => {
					let event = Event::MouseLeave;
					builder
						.process_input
						.iter()
						.for_each(|process| process(engine, &event));
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

					builder.tick.iter().for_each(|tick| tick(engine, dt));
				}
				WinitEvent::RedrawRequested(_) => match &builder.display {
					Some(display) => (display)(engine),
					None => {}
				},
				_ => (),
			}

			// Do pre shutdowns
			if *control_flow == ControlFlow::Exit {
				builder
					.pre_shutdown
					.drain(..)
					.for_each(|shutdown| shutdown(engine));
			}
		});
	}

	/// Returns the global [`Engine`] as a ref
	pub fn as_ref() -> &'static Engine {
		unsafe { ENGINE.as_ref().unwrap() }
	}

	pub fn module<'a, T: Module>(&'a self) -> Option<&'a T> {
		let id = TypeId::of::<T>();

		let module = self.modules.get(&id)?;
		module.downcast_ref::<T>()
	}

	pub fn register<T: Register>(&self) -> Option<Vec<T>> {
		let id = TypeId::of::<T>();

		let register = self.registers.get(&id)?;
		Some(register.downcast_ref::<Vec<T>>()?.clone())
	}

	/// Returns the name of the engine runnable
	pub fn name(&self) -> &str {
		&self.name
	}

	/// Returns the window that the engine draws into
	pub fn window(&self) -> Option<&Window> {
		self.window.as_ref()
	}

	pub fn shutdown(&self) {
		self.is_running.store(false, Ordering::Relaxed);
	}

	pub fn fps(&self) -> i32 {
		self.fps.load(Ordering::Relaxed)
	}
}
