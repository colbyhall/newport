mod editor;
mod render;
pub(crate) use {
	editor::*,
	render::*,
};

use {
	ecs::{
		Component,
		Ecs,
		ScheduleBlock,
		World,
	},
	engine::{
		Builder,
		Engine,
		Module,
	},
	graphics::Graphics,
	math::*,
	resources::Handle,
	serde::{
		Deserialize,
		Serialize,
	},
	std::cell::UnsafeCell,
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Transform {
	pub location: Point3,
	pub rotation: Quaternion,
	pub scale: Vec3,
}

impl Transform {
	pub fn to_mat4(&self) -> Mat4 {
		// TODO: Do this without mat4 multiplication
		Mat4::rotate(self.rotation) * Mat4::translate(self.location)
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			location: Point3::ZERO,
			rotation: Quaternion::IDENTITY,
			scale: Vec3::ONE,
		}
	}
}

pub struct Game {
	world: World,
	renderer: Renderer,

	viewport: UnsafeCell<Vec2>,
}
impl Module for Game {
	fn new() -> Self {
		let world = World::new(ScheduleBlock::new());
		{
			let mut transforms = world.write::<Transform>();
			let mut filters = world.write::<MeshFilter>();
			let mut cameras = world.write::<Camera>();

			let pipeline =
				Handle::find_or_load("{D0FAF8AC-0650-48D1-AAC2-E1C01E1C93FC}").unwrap_or_default();
			world
				.spawn()
				.with(Transform::default(), &mut transforms)
				.with(Camera::default(), &mut cameras)
				.finish();

			world
				.spawn()
				.with(
					Transform {
						location: Point3::new(5.0, 0.0, -1.0),
						..Default::default()
					},
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

			viewport: UnsafeCell::new(viewport),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<Ecs>()
			.module::<Editor>()
			.register(Camera::variant())
			.register(MeshFilter::variant())
			.register(Transform::variant())
			.tick(|delta_time| {
				let Game {
					world,
					renderer,
					viewport,
					..
				} = Engine::module().unwrap();

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
			})
	}
}
