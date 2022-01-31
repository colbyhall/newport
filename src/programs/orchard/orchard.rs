use {
	ecs::{
		Component,
		Ecs,
		Query,
		ScheduleBlock,
		System,
		World,
	},
	engine::{
		define_run_module,
		input::*,
		Builder,
		Engine,
		Event,
		Module,
	},
	gpu::{
		Buffer,
		BufferUsage,
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
		Layout::*,
		MemoryType::*,
	},
	graphics::{
		Graphics,
		Painter,
	},
	math::{
		Color,
		Mat4,
		Point2,
		Rect,
		Vec2,
	},
	resources::{
		Handle,
		ResourceManager,
	},
	serde::{
		Deserialize,
		Serialize,
	},
	std::collections::HashMap,
};

pub struct Game {
	world: World,
	input_queue: Vec<Event>,

	pipeline: Handle<GraphicsPipeline>,
}
impl Module for Game {
	fn new() -> Self {
		let world = World::new(None, ScheduleBlock::new());
		{
			let mut transforms = world.write::<Transform>();
			let mut box_colliders = world.write::<BoxCollider>();
			let mut cameras = world.write::<Camera>();

			// Test Block
			world
				.spawn(world.persistent)
				.with(
					Transform {
						location: Vec2::new(0.0, 1.0),
						..Default::default()
					},
					&mut transforms,
				)
				.with(BoxCollider::default(), &mut box_colliders)
				.finish();

			// Floor
			world
				.spawn(world.persistent)
				.with(Transform::default(), &mut transforms)
				.with(
					BoxCollider {
						size: Vec2::new(500.0, 1.0),
					},
					&mut box_colliders,
				)
				.finish();

			world
				.spawn(world.persistent)
				.with(Transform::default(), &mut transforms)
				.with(Camera::default(), &mut cameras)
				.finish();
		}

		Self {
			world,
			input_queue: Vec::with_capacity(256),
			pipeline: Handle::find_or_load("{03996604-84B2-437D-98CA-A816D7768DCB}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Ecs>()
			.module::<Graphics>()
			.module::<ResourceManager>()
			.register(Transform::variant())
			.register(Camera::variant())
			.register(BoxCollider::variant())
			.register(InputManager::variant())
			.process_input(|event| {
				let game: &mut Game = unsafe { Engine::module_mut().unwrap() };
				game.input_queue.push(*event);
			})
			.tick(|dt| {
				let game: &Game = Engine::module().unwrap();
				game.world.step(dt);
			})
			.display(|| {
				let game: &Game = Engine::module().unwrap();
				let Game {
					world, pipeline, ..
				} = game;

				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();
				let aspect_ratio = (backbuffer.width() as f32) / (backbuffer.height() as f32);

				let transforms = world.read::<Transform>();
				let box_colliders = world.read::<BoxCollider>();
				let cameras = world.read::<Camera>();

				let entities = Query::new().read(&cameras).read(&transforms).execute(world);
				let view = if let Some(e) = entities.iter().cloned().next() {
					let transform = transforms.get(e).unwrap();
					let camera = cameras.get(e).unwrap();

					let proj = Mat4::ortho(camera.size * aspect_ratio, camera.size, 1000.0, 0.1);
					Some(proj * Mat4::translate((-transform.location, 0.0)))
				} else {
					None
				};

				let entities = Query::new()
					.read(&transforms)
					.read(&box_colliders)
					.execute(world);

				let mut painter = Painter::new();
				for e in entities.iter().copied() {
					let transform = transforms.get(e).unwrap();
					let collider = box_colliders.get(e).unwrap();

					painter.fill_rect(
						Rect::from_center(transform.location, collider.size),
						Color::WHITE,
					);
				}
				if view.is_none() {
					todo!("No Camera Debug Text");
				}
				let (vertices, indices) = painter.finish().unwrap();

				#[allow(dead_code)]
				struct Imports {
					view: Mat4,
				}
				let imports = Buffer::new(BufferUsage::CONSTANTS, HostVisible, 1).unwrap();
				imports
					.copy_to(&[Imports {
						view: view.unwrap_or(Mat4::IDENTITY),
					}])
					.unwrap();

				let pipeline = pipeline.read();

				let receipt = GraphicsRecorder::new()
					.texture_barrier(&backbuffer, Undefined, ColorAttachment)
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.set_pipeline(&pipeline)
							.set_vertex_buffer(&vertices)
							.set_index_buffer(&indices)
							.set_constants("imports", &imports, 0)
							.draw_indexed(indices.len(), 0)
					})
					.texture_barrier(&backbuffer, ColorAttachment, Present)
					.submit();
				device.display(&[receipt]);
			})
	}
}

define_run_module!(Game, "Orchard");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
	location: Point2,
	layer: u32,
	rotation: f32,
	scale: Vec2,
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			location: Point2::ZERO,
			layer: 0,
			rotation: 0.0,
			scale: Vec2::ONE,
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BoxCollider {
	size: Vec2,
}

impl Default for BoxCollider {
	fn default() -> Self {
		Self { size: Vec2::ONE }
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Camera {
	size: f32,
}

impl Default for Camera {
	fn default() -> Self {
		Self { size: 20.0 }
	}
}

#[derive(Clone, Copy, Debug)]
pub enum InputState {
	Button(bool),
	Axis1D(f32),
}

impl InputState {
	pub fn button(self) -> bool {
		match self {
			Self::Button(b) => b,
			_ => unreachable!(),
		}
	}

	pub fn axis1d(self) -> f32 {
		match self {
			Self::Axis1D(x) => x,
			_ => unreachable!(),
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InputManager {
	#[serde(skip)]
	current: HashMap<Input, InputState>,
	#[serde(skip)]
	last: HashMap<Input, InputState>,
}

impl InputManager {
	pub fn is_button_down(&self, button: Input) -> bool {
		match self.current.get(&button) {
			Some(input_state) => input_state.button(),
			None => false,
		}
	}

	pub fn was_button_pressed(&self, button: Input) -> bool {
		let down = match self.current.get(&button) {
			Some(input_state) => input_state.button(),
			None => false,
		};
		let last = match self.last.get(&button) {
			Some(input_state) => input_state.button(),
			None => false,
		};

		!last && down
	}

	pub fn was_button_released(&self, button: Input) -> bool {
		let down = match self.current.get(&button) {
			Some(input_state) => input_state.button(),
			None => false,
		};
		let last = match self.last.get(&button) {
			Some(input_state) => input_state.button(),
			None => false,
		};

		last && !down
	}

	pub fn current_axis1d(&self, axis: Input) -> f32 {
		match self.current.get(&axis) {
			Some(input_state) => input_state.axis1d(),
			None => 0.0,
		}
	}

	pub fn last_axis1d(&self, axis: Input) -> f32 {
		match self.last.get(&axis) {
			Some(input_state) => input_state.axis1d(),
			None => 0.0,
		}
	}

	pub fn delta_axis1d(&self, axis: Input) -> f32 {
		self.current_axis1d(axis) - self.last_axis1d(axis)
	}
}

#[derive(Clone)]
pub struct InputSystem;
impl System for InputSystem {
	fn run(&self, world: &World, _dt: f32) {
		// Lazy load the input manager component
		let mut input_managers = world.write::<InputManager>();
		let input_manager = match input_managers.get_mut(world.singleton) {
			Some(c) => c,
			None => {
				world.insert(
					&mut input_managers,
					world.singleton,
					InputManager::default(),
				);
				input_managers.get_mut(world.singleton).unwrap()
			}
		};

		// Swap current state to last for new current state
		input_manager.last = input_manager.current.clone();

		let InputManager { current, .. } = input_manager;

		// Reset all current axis input to 0
		for (_, value) in current.iter_mut() {
			if let InputState::Axis1D(x) = value {
				*x = 0.0;
			}
		}

		// Process input into state maps
		let game: &Game = Engine::module().unwrap();
		for event in game.input_queue.iter() {
			match event {
				Event::Key { key, pressed } => match current.get_mut(key) {
					Some(input_state) => {
						if let InputState::Button(b) = input_state {
							*b = *pressed;
						} else {
							unreachable!();
						}
					}
					None => {
						current.insert(*key, InputState::Button(*pressed));
					}
				},
				Event::MouseButton {
					mouse_button,
					pressed,
				} => match current.get_mut(mouse_button) {
					Some(input_state) => {
						if let InputState::Button(b) = input_state {
							*b = *pressed;
						} else {
							unreachable!();
						}
					}
					None => {
						current.insert(*mouse_button, InputState::Button(*pressed));
					}
				},
				Event::MouseMotion(x, y) => {
					let mut set_motion = |input, value| match current.get_mut(&input) {
						Some(input_state) => {
							if let InputState::Axis1D(x) = input_state {
								*x = value;
							} else {
								unreachable!();
							}
						}
						None => {
							current.insert(input, InputState::Axis1D(value));
						}
					};
					if *x != 0.0 {
						set_motion(MOUSE_AXIS_X, *x);
					}
					if *y != 0.0 {
						set_motion(MOUSE_AXIS_Y, *y);
					}
				}
				_ => {}
			}
		}
	}
}
