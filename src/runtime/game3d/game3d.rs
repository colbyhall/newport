mod render;
pub(crate) use render::*;

pub use render::{
	Camera,
	DebugManager,
	DebugSystem,
	DirectionalLight,
	Mesh,
	MeshFilter,
};
use resources::Importer;

use {
	draw2d::Draw2d,
	ecs::{
		Component,
		Ecs,
		Entity,
		Query,
		ScheduleBlock,
		System,
		World,
		WriteStorage,
	},
	engine::{
		Builder,
		Engine,
		Module,
	},
	input::*,
	math::*,
	resources::Resource,
	serde::{
		Deserialize,
		Serialize,
	},
	std::sync::Mutex,
};

#[cfg(feature = "editor")]
use editor::Editor;

#[cfg(not(feature = "editor"))]
use {
	gpu::{
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
	},
	resources::Handle,
};

pub struct Game {
	pub world: World,
	pub schedule: Mutex<ScheduleBlock>,
	renderer: Renderer,

	#[cfg(not(feature = "editor"))]
	present_pipeline: Handle<GraphicsPipeline>,
}
impl Module for Game {
	fn new() -> Self {
		Self {
			world: World::new(None),
			schedule: Mutex::new(ScheduleBlock::new()),
			renderer: Renderer::new(),

			#[cfg(not(feature = "editor"))]
			present_pipeline: Handle::find_or_load("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}")
				.unwrap(),
		}
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<Draw2d>()
			.module::<Ecs>()
			.module::<GameInput>();

		#[cfg(feature = "editor")]
		builder.module::<Editor>();

		builder
			.register(Transform::variant())
			.register(Camera::variant())
			.register(Mesh::variant())
			.register(MeshGltfImporter::variant(&["gltf", "glb"]))
			.register(MeshFilter::variant())
			.register(EditorCameraController::variant())
			.register(DebugManager::variant())
			.register(DirectionalLight::variant())
			.tick(|delta_time| {
				let Game {
					world,
					renderer,
					schedule,
					..
				} = Engine::module().unwrap();

				let window = Engine::window().unwrap();
				let viewport = window.inner_size();
				let viewport = Vec2::new(viewport.width as f32, viewport.height as f32);

				{
					{
						let schedule = schedule.lock().unwrap();
						schedule.execute(world, delta_time);
						let scene = DrawList::build(world, viewport);
						renderer.push_scene(scene);
					}
					renderer.render_scene();
				};
				renderer.advance_frame();
			});

		#[cfg(not(feature = "editor"))]
		builder.display(|| {
			let game: &Game = Engine::module().unwrap();

			let device = Gpu::device();
			let backbuffer = device
				.acquire_backbuffer()
				.expect("Swapchain failed to find a back buffer");

			let pipeline = game.present_pipeline.read();
			let receipt = match game.renderer.to_display() {
				Some(scene) => GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.set_pipeline(&pipeline)
							.set_texture("texture", &scene.diffuse_buffer)
							.draw(3, 0)
					})
					.texture_barrier(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.submit(),
				None => GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::BLACK))
					.texture_barrier(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.submit(),
			};

			device.display(&[receipt]);
		});

		builder
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
		let mut result = Mat4::IDENTITY;
		result.x_column = (self.rotation.rotate(Vec3::FORWARD) * self.scale.x, 0.0).into();
		result.y_column = (self.rotation.rotate(Vec3::RIGHT) * self.scale.y, 0.0).into();
		result.z_column = (self.rotation.rotate(Vec3::UP) * self.scale.z, 0.0).into();
		result.w_column = (self.location, 1.0).into();

		result
	}

	pub fn world_mat4(&self) -> Mat4 {
		if self.parent.is_some() {
			self.local_to_world * self.local_mat4()
		} else {
			self.local_mat4()
		}
	}

	pub fn local_location(&self) -> Vec3 {
		self.location
	}

	// TODO: Figure out best api for local and world location. Also marking as changed
	pub fn set_local_location(&mut self, location: impl Into<Vec3>) -> &mut Self {
		self.location = location.into();
		self.changed = true;
		self
	}

	pub fn local_rotation(&self) -> Quat {
		self.rotation
	}

	pub fn set_local_rotation(&mut self, rotation: Quat) -> &mut Self {
		self.rotation = rotation;
		self.changed = true;
		self
	}

	pub fn changed(&self) -> bool {
		self.changed
	}

	pub fn set_changed(&mut self, changed: bool) -> &mut Self {
		self.changed = changed;
		self
	}
}

impl Component for Transform {
	fn on_added(_world: &World, entity: Entity, storage: &mut WriteStorage<Self>) {
		let mut child = storage.get_mut(entity).unwrap();

		if let Some(parent) = child.parent {
			let parent = storage.get(parent).expect("Parent should have a transform");
			child.local_to_world = parent.local_to_world * parent.local_mat4();
			child.world_to_local = child.local_to_world.inverse().unwrap_or_default();
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EditorCameraController {
	pub pitch: f32,
	pub yaw: f32,

	pub use_mouse_input: bool,
	pub is_sprinting: bool,
}

impl Component for EditorCameraController {}

#[derive(Clone)]
pub struct EditorCameraSystem;
impl System for EditorCameraSystem {
	fn run(&self, world: &World, dt: f32) {
		let input = world.read::<InputManager>();
		let input = input.get(world.singleton).unwrap();

		// Query for all controllers that could be functioning
		let transforms = world.write::<Transform>();
		let controllers = world.write::<EditorCameraController>();
		let cameras = world.read::<Camera>();
		let entities = Query::new()
			.write(&transforms)
			.write(&controllers)
			.read(&cameras)
			.execute(world);

		// Essentially all we're doing is handling inputs and updating transforms
		for e in entities.iter().copied() {
			let mut transform = transforms.get_mut(e).unwrap();
			let mut controller = controllers.get_mut(e).unwrap();

			const MOUSE_INPUT_TOGGLE_KEY: Input = KEY_ESCAPE;
			if input.was_button_pressed(MOUSE_INPUT_TOGGLE_KEY) {
				controller.use_mouse_input = !controller.use_mouse_input;

				let window = Engine::window().unwrap();

				// TODO: This should be part of some general viewport abstraction
				if controller.use_mouse_input {
					window.set_cursor_grab(true).unwrap();
					window.set_cursor_visible(false);
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
