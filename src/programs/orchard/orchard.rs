use {
	ecs::{
		Ecs,
		ScheduleBlock,
		World,
	},
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Gpu,
		GraphicsRecorder,
		Layout::*,
	},
	graphics::Graphics,
	math::{
		Color,
		Point2,
		Vec2,
	},
	resources::ResourceManager,
	serde::{
		Deserialize,
		Serialize,
	},
};

pub struct Game {
	world: World,
}
impl Module for Game {
	fn new() -> Self {
		let world = World::new(None, ScheduleBlock::new());
		Self { world }
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Ecs>()
			.module::<Graphics>()
			.module::<ResourceManager>()
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();
				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| ctx.clear_color(Color::RED))
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
		Self { size: 400.0 }
	}
}
