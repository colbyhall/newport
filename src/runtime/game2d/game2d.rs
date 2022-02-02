use {
	ecs::{
		Component,
		Ecs,
		Entity,
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
		Texture,
	},
	graphics::{
		Graphics,
		Painter,
	},
	input::*,
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
};

pub struct Game {
	world: World,
	pipeline: Handle<GraphicsPipeline>,
}
impl Module for Game {
	fn new() -> Self {
		let world = World::new(
			None,
			ScheduleBlock::new()
				.system(InputSystem)
				.system(PlayerControlledMovement)
				.system(CharacterMovementSystem)
				.system(CameraTracking),
		);

		Self {
			world,
			pipeline: Handle::find_or_load("{03996604-84B2-437D-98CA-A816D7768DCB}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Ecs>()
			.module::<Graphics>()
			.module::<ResourceManager>()
			.module::<GameInput>()
			.register(Transform::variant())
			.register(Camera::variant())
			.register(Sprite::variant())
			.register(PlayerControlled::variant())
			.register(CharacterMovement::variant())
			.register(Target::variant())
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
				let sprites = world.read::<Sprite>();
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

				let entities = Query::new().read(&transforms).read(&sprites).execute(world);

				let mut painter = Painter::new();
				for e in entities.iter().copied() {
					let transform = transforms.get(e).unwrap();
					let sprite = sprites.get(e).unwrap();

					match &sprite.texture {
						None => painter.fill_rect(
							Rect::from_center(transform.location, sprite.extents),
							sprite.color,
						),
						_ => unimplemented!(),
					};
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
pub struct Sprite {
	texture: Option<Handle<Texture>>,
	color: Color,
	uv: Rect,
	pipeline: Handle<GraphicsPipeline>,
	extents: Vec2,
}

impl Default for Sprite {
	fn default() -> Self {
		Self {
			texture: None, // TODO: Default sprite texture????
			color: Color::WHITE,
			uv: Rect::from_min_max((0.0, 0.0), (1.0, 1.0)),
			pipeline: Handle::find_or_load("{03996604-84B2-437D-98CA-A816D7768DCB}").unwrap(),
			extents: Vec2::splat(1.0),
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Camera {
	size: f32,
}

impl Default for Camera {
	fn default() -> Self {
		Self { size: 10.0 }
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlayerControlled;

#[derive(Clone)]
pub struct PlayerControlledMovement;
impl System for PlayerControlledMovement {
	fn run(&self, world: &World, _dt: f32) {
		let input = world.read::<InputManager>();
		let input = input.get(world.singleton).unwrap();

		let controllers = world.read::<PlayerControlled>();
		let mut character_movements = world.write::<CharacterMovement>();

		let entities = Query::new()
			.read(&controllers)
			.write(&character_movements)
			.execute(world);

		for e in entities.iter().copied() {
			let character_movement = character_movements.get_mut(e).unwrap();

			let mut new_input = Vec2::ZERO;
			if input.is_button_down(KEY_A) {
				new_input.x = -1.0;
			}
			if input.is_button_down(KEY_D) {
				new_input.x = 1.0;
			}
			character_movement.jump_pressed = input.was_button_pressed(KEY_SPACE);
		}
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CharacterMovement {
	last_input: Option<Vec2>,
	jump_pressed: bool,
	velocity: Vec2,
}

#[derive(Clone)]
pub struct CharacterMovementSystem;
impl System for CharacterMovementSystem {
	fn run(&self, _world: &World, _dt: f32) {}
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Target {
	entity: Option<Entity>,
}

#[derive(Clone)]
pub struct CameraTracking;
impl System for CameraTracking {
	fn run(&self, world: &World, dt: f32) {
		let mut transforms = world.write::<Transform>();
		let cameras = world.read::<Camera>();
		let targets = world.read::<Target>();

		const SPEED: f32 = 5.0;

		let entities = Query::new()
			.write(&transforms)
			.read(&cameras)
			.read(&targets)
			.execute(world);

		for e in entities.iter().cloned() {
			let target = targets.get(e).unwrap();
			if let Some(target) = &target.entity {
				if let Some(target) = transforms.get(*target).cloned() {
					let transform = transforms.get_mut(e).unwrap();
					transform.location =
						Vec2::lerp(transform.location, target.location, dt * SPEED);
				}
			}
		}
	}
}
