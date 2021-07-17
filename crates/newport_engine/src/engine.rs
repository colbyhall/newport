use crate::{
	math::Rect,
	os::window::{
		WindowBuilder,
		WindowStyle,
	},

	Builder,
	Module,
	Register,
};

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
	process,
	sync::atomic::{
		AtomicBool,
		Ordering,
	},
	sync::Mutex,
	time::Instant,
};

pub use crate::os::window::{
	Window,
	WindowEvent as InputEvent,
};

static mut ENGINE: Option<Engine> = None;

/// Global runnable structure used for instantiating engine modules and handling app code
///
/// Created using an [`Builder`] which defines the functionality of the app using [`Module`]s
pub struct Engine {
	name: String,
	modules: HashMap<TypeId, Box<dyn Any>>,
	registers: HashMap<TypeId, Box<dyn Any>>,

	is_running: AtomicBool,
	fps: i32,

	window: Window,
	minimize: AtomicBool,
	maximize: AtomicBool,
	drag: Mutex<Rect>,
	dpi: f32,
}

impl Engine {
	/// Starts the engine using what was built with a [`Builder`]
	///
	/// # Arguments
	///
	/// * `builder` - An [`Builder`] used to setup app execution and structure
	///
	/// # Examples
	///
	/// ```
	/// use newport_engine::{ Builder, Engine };
	/// use newport_asset::AssetManager;
	///
	/// let builder = Builder::new()
	///     .module::<AssetManager>();
	/// Engine::run(builder).unwrap();
	/// ```
	pub(crate) fn run(mut builder: Builder) {
		// Grab the project name or use a default
		let name = builder.name.unwrap_or("newport".to_string());

		// UNSAFE: Set the global state
		let engine = unsafe {
			let id = TypeId::of::<WindowStyle>();
			let styles: Vec<WindowStyle> = match builder.registers.get(&id) {
				Some(any_vec) => any_vec.downcast_ref::<Vec<WindowStyle>>().unwrap().clone(),
				None => Vec::default(),
			};
			let style = match styles.last() {
				Some(style) => *style,
				None => WindowStyle::Windowed,
			};

			let window = WindowBuilder::new()
				.title(name.clone())
				.style(style)
				.spawn()
				.unwrap();

			let dpi = window.dpi();

			ENGINE = Some(Engine {
				name: name,
				modules: HashMap::with_capacity(builder.entries.len()),
				registers: builder.registers,

				is_running: AtomicBool::new(true),
				fps: 0,

				window: window,
				minimize: AtomicBool::new(false),
				maximize: AtomicBool::new(false),

				drag: Mutex::new(Rect::default()),
				dpi: dpi,
			});

			ENGINE.as_mut().unwrap()
		};

		// NOTE: All modules a module depends on will be available at initialization
		builder.entries.drain(..).for_each(|it| {
			engine.modules.insert(it.id, (it.spawn)());
		});

		// Do post init
		builder.post_inits.drain(..).for_each(|init| init(engine));

		engine.window.set_visible(true);
		engine.window.maximize();

		let mut frame_count = 0;
		let mut time = 0.0;

		// Game loop
		let mut last_frame_time = Instant::now();
		'run: while engine.is_running.load(Ordering::Relaxed) {
			let now = Instant::now();
			let dt = now.duration_since(last_frame_time).as_secs_f32();
			last_frame_time = now;

			time += dt;
			if time >= 1.0 {
				time = 0.0;
				engine.fps = frame_count;
				frame_count = 0;
			}
			frame_count += 1;

			{
				for event in engine.window.poll_events() {
					builder
						.process_input
						.iter()
						.for_each(|process_input| process_input(engine, &engine.window, &event));

					match event {
						InputEvent::Closed => {
							engine.is_running.store(false, Ordering::Relaxed);
							break 'run;
						}
						InputEvent::Resizing(_, _) => {
							builder.tick.iter().for_each(|tick| tick(engine, 0.0));
						}
						_ => {}
					}
				}
			}

			builder.tick.iter().for_each(|tick| tick(engine, dt));

			if engine.maximize.load(Ordering::Relaxed) {
				engine.window.maximize();
				engine.maximize.store(false, Ordering::Relaxed)
			}

			if engine.minimize.load(Ordering::Relaxed) {
				engine.window.minimize();
				engine.minimize.store(false, Ordering::Relaxed)
			}

			{
				let drag = engine.drag.lock().unwrap();
				engine.window.set_custom_drag(*drag);
			}
		}

		// Do pre shutdowns
		builder
			.pre_shutdown
			.drain(..)
			.for_each(|shutdown| shutdown(engine));

		process::exit(0);
	}

	/// Returns the global [`Engine`] as a ref
	pub fn as_ref() -> &'static Engine {
		unsafe { ENGINE.as_ref().unwrap() }
	}

	/// Searches a module by type and returns an [`Option<&'static T>`]
	///
	/// # Arguments
	///
	/// * `T` - A [`Module`] that should have been created using a [`Builder`]
	///
	/// # Examples
	///
	/// ```
	/// use newport_engine::Engine;
	///
	/// let engine = Engine::as_ref();
	/// let module = engine.module::<Module>().unwrap();
	/// ```
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
	pub fn window(&self) -> &Window {
		&self.window
	}

	pub fn shutdown(&self) {
		self.is_running.store(false, Ordering::Relaxed);
	}

	pub fn fps(&self) -> i32 {
		self.fps
	}

	pub fn maximize(&self) {
		self.maximize.store(true, Ordering::Relaxed);
	}

	pub fn minimize(&self) {
		self.minimize.store(true, Ordering::Relaxed);
	}

	pub fn set_custom_drag(&self, drag: Rect) {
		let mut self_drag = self.drag.lock().unwrap();
		*self_drag = drag;
	}

	pub fn dpi(&self) -> f32 {
		self.dpi
	}
}
