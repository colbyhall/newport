mod editor;
mod render;
use ecs::OnAdded;
pub use os::input::*;
pub(crate) use {
	editor::*,
	render::*,
};

pub use render::{
	Camera,
	DebugManager,
	MeshFilter,
};

use {
	ecs::{
		Component,
		Ecs,
		Entity,
		Named,
		Query,
		ScheduleBlock,
		System,
		World,
	},
	engine::{
		Builder,
		Engine,
		Event,
		Module,
	},
	graphics::Graphics,
	math::*,
	resources::Handle,
	serde::{
		Deserialize,
		Serialize,
	},
	std::{
		cell::UnsafeCell,
		collections::HashMap,
	},
};

pub struct Game {
	world: World,
	renderer: Renderer,
	input_queue: Vec<Event>,

	viewport: UnsafeCell<Vec2>,
}
impl Module for Game {
	fn new() -> Self {
		let world = World::new(
			None,
			ScheduleBlock::new()
				.system(InputSystem)
				.system(DebugSystem)
				.system(EditorCameraSystem),
		);
		{
			let mut transforms = world.write::<Transform>();
			let mut filters = world.write::<MeshFilter>();
			let mut cameras = world.write::<Camera>();
			let mut names = world.write::<Named>();
			let mut camera_controllers = world.write::<EditorCameraController>();

			let pipeline =
				Handle::find_or_load("{D0FAF8AC-0650-48D1-AAC2-E1C01E1C93FC}").unwrap_or_default();
			world
				.spawn(world.persistent)
				.with(Transform::default(), &mut transforms)
				.with(Camera::default(), &mut cameras)
				.with(Named::new("Camera"), &mut names)
				.with(EditorCameraController::default(), &mut camera_controllers)
				.finish();

			world
				.spawn(world.persistent)
				.with(
					Transform::builder()
						.location(Vec3::new(5.0, 5.0, 5.0))
						// .scale(Vec3::splat(0.2))
						.finish(),
					&mut transforms,
				)
				.with(
					MeshFilter {
						mesh: Handle::find_or_load("{03383b92-566f-4036-aeb4-850b61685ea6}")
							.unwrap_or_default(),
						pipeline: pipeline.clone(),
					},
					&mut filters,
				)
				.with(Named::new("Block"), &mut names)
				.finish();

			let parent = world
				.spawn(world.persistent)
				.with(
					Transform::builder()
						.location([5.0, 0.0, 0.0])
						.rotation(Quat::from_euler([45.0, 45.0, 0.0]))
						.scale(Vec3::splat(0.2))
						.finish(),
					&mut transforms,
				)
				.with(
					MeshFilter {
						mesh: Handle::find_or_load("{03383b92-566f-4036-aeb4-850b61685ea6}")
							.unwrap_or_default(),
						pipeline: pipeline.clone(),
					},
					&mut filters,
				)
				.finish();

			world
				.spawn(world.persistent)
				.with(
					Transform::builder()
						.location([5.0, 0.0, 0.0])
						.rotation(Quat::from_euler([45.0, 45.0, 0.0]))
						.parent(parent)
						.finish(),
					&mut transforms,
				)
				.with(
					MeshFilter {
						mesh: Handle::find_or_load("{03383b92-566f-4036-aeb4-850b61685ea6}")
							.unwrap_or_default(),
						pipeline,
					},
					&mut filters,
				)
				.finish();
		}

		let window = Engine::window().unwrap();
		let viewport = window.inner_size();
		let viewport = Vec2::new(viewport.width as f32, viewport.height as f32);

		Self {
			world,
			renderer: Renderer::new(),
			input_queue: Vec::with_capacity(64),

			viewport: UnsafeCell::new(viewport),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<Ecs>()
			.module::<Editor>()
			.register(Transform::variant().on_added::<Transform>())
			.register(Camera::variant())
			.register(MeshFilter::variant())
			.register(InputManager::variant())
			.register(EditorCameraController::variant())
			.register(DebugManager::variant())
			.tick(|delta_time| {
				let Game {
					world,
					renderer,
					viewport,
					input_queue,
					..
				} = unsafe { Engine::module_mut().unwrap() };

				let viewport = {
					let viewport = viewport.get();
					unsafe { *viewport }
				};

				{
					{
						world.step(delta_time);
						let scene = DrawList::build(world, viewport);
						renderer.push_scene(scene);
					}
					renderer.render_scene();
				};
				renderer.advance_frame();

				input_queue.clear(); // UNSAFE: This shouldnt change on other threads
			})
			.process_input(|event| {
				let game: &mut Game = unsafe { Engine::module_mut().unwrap() };
				game.input_queue.push(*event);
			})
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
	location: Point3,
	rotation: Quat,
	scale: Vec3,

	parent: Option<Entity>,
	children: Vec<Entity>,

	// Cached data
	changed: bool,
	local_to_world: Mat4,
	world_to_local: Mat4,
}

impl Transform {
	pub fn builder() -> TransformBuilder {
		TransformBuilder {
			transform: Transform::default(),
		}
	}

	pub fn local_mat4(&self) -> Mat4 {
		// TODO: Do this without mat4 multiplication
		Mat4::translate(self.location) * Mat4::rotate(self.rotation) * Mat4::scale(self.scale)
	}
}

pub struct TransformBuilder {
	transform: Transform,
}

impl TransformBuilder {
	#[must_use]
	pub fn location(mut self, location: impl Into<Point3>) -> Self {
		self.transform.location = location.into();
		self
	}

	#[must_use]
	pub fn rotation(mut self, rotation: Quat) -> Self {
		self.transform.rotation = rotation;
		self
	}

	#[must_use]
	pub fn scale(mut self, scale: impl Into<Vec3>) -> Self {
		self.transform.scale = scale.into();
		self
	}

	#[must_use]
	pub fn parent(mut self, entity: Entity) -> Self {
		self.transform.parent = Some(entity);
		self
	}

	pub fn finish(self) -> Transform {
		self.transform
	}
}

impl OnAdded for Transform {
	fn on_added(entity: Entity, storage: &mut ecs::AnyWriteStorage) {
		// We need to ensure the parent is aware we exist and update all the cache data
		let parent = storage.get::<Transform>(entity).unwrap().parent;
		if let Some(parent) = parent {
			let transform: &mut Transform = storage.get_mut(parent).unwrap();
			transform.children.push(entity);

			let local_to_world = transform.local_to_world * transform.local_mat4();
			let transform: &mut Transform = storage.get_mut(entity).unwrap();
			transform.local_to_world = local_to_world;
			transform.world_to_local = local_to_world.inverse().unwrap();
		}
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			location: Point3::ZERO,
			rotation: Quat::IDENTITY,
			scale: Vec3::ONE,

			parent: None,
			children: Vec::with_capacity(32),

			changed: false,
			local_to_world: Mat4::IDENTITY,
			world_to_local: Mat4::IDENTITY,
		}
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
		for e in game.input_queue.iter() {
			match e {
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
						set_motion(os::MOUSE_AXIS_X, *x);
					}
					if *y != 0.0 {
						set_motion(os::MOUSE_AXIS_Y, *y);
					}
				}
				_ => {}
			}
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EditorCameraController {
	pub pitch: f32,
	pub yaw: f32,

	pub use_mouse_input: bool,
	pub is_sprinting: bool,
}

#[derive(Clone)]
pub struct EditorCameraSystem;
impl System for EditorCameraSystem {
	fn run(&self, world: &World, dt: f32) {
		let input = world.read::<InputManager>();
		let input = input.get(world.singleton).unwrap();

		// Query for all controllers that could be functioning
		let mut transforms = world.write::<Transform>();
		let mut controllers = world.write::<EditorCameraController>();
		let cameras = world.read::<Camera>();
		let entities = Query::new()
			.write(&transforms)
			.write(&controllers)
			.read(&cameras)
			.execute(world);

		// Essentially all we're doing is handling inputs and updating transforms
		for e in entities.iter().copied() {
			let transform = transforms.get_mut(e).unwrap();
			let controller = controllers.get_mut(e).unwrap();

			const MOUSE_INPUT_TOGGLE_KEY: Input = KEY_ESCAPE;
			if input.was_button_pressed(MOUSE_INPUT_TOGGLE_KEY) {
				controller.use_mouse_input = !controller.use_mouse_input;

				let window = Engine::window().unwrap();

				// TODO: This should be part of some general viewport abstraction
				if controller.use_mouse_input {
					window.set_cursor_grab(true).unwrap();
					window.set_cursor_visible(false);

					let size = window.outer_size();

					window
						.set_cursor_position(os::winit::dpi::PhysicalPosition::new(
							size.width / 2,
							size.height / 2,
						))
						.unwrap();
				} else {
					window.set_cursor_grab(false).unwrap();
					window.set_cursor_visible(true);
				}
			}

			// Update the camera controller rotation only when mouse input is being consumed
			if controller.use_mouse_input {
				const SENSITIVITY: f32 = 0.3;
				controller.pitch -= input.current_axis1d(MOUSE_AXIS_Y) * SENSITIVITY;
				controller.yaw += input.current_axis1d(MOUSE_AXIS_X) * SENSITIVITY;
				transform.rotation =
					Quat::from_euler(Vec3::new(controller.pitch, controller.yaw, 0.0));
			}

			// Determine the current movement speed
			const WALK_SPEED: f32 = 6.0;
			const SPRINT_SPEED: f32 = 20.0;
			let speed = if controller.is_sprinting {
				SPRINT_SPEED
			} else {
				WALK_SPEED
			};

			// Move camera forward and right axis. Up and down on world UP
			let forward = transform.rotation.forward();
			let right = transform.rotation.right();
			let up = Vec3::UP;
			if input.is_button_down(KEY_W) {
				transform.location += forward * dt * speed;
			}
			if input.is_button_down(KEY_S) {
				transform.location -= forward * dt * speed;
			}
			if input.is_button_down(KEY_D) {
				transform.location += right * dt * speed;
			}
			if input.is_button_down(KEY_A) {
				transform.location -= right * dt * speed;
			}
			if input.is_button_down(KEY_SPACE) {
				transform.location += up * dt * speed;
			}
			if input.is_button_down(KEY_LCTRL) {
				transform.location -= up * dt * speed;
			}
		}
	}
}
