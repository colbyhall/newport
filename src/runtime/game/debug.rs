use {
	ecs::{
		Component,
		System,
		World,
	},
	gpu::GraphicsPipeline,
	math::{
		Color,
		Point3,
		Quat,
		Vec3,
	},
	resources::Handle,
	serde::{
		Deserialize,
		Serialize,
	},
};

#[derive(Clone, Debug)]
pub struct DebugShape {
	pub line_width: f32,
	pub color: Color,

	pub time_left: f32,

	pub location: Point3,
	pub rotation: Quat,
	pub variant: DebugShapeVariant,
}

impl DebugShape {
	pub fn color(&mut self, color: Color) -> &mut Self {
		self.color = color;
		self
	}

	pub fn line_width(&mut self, line_width: f32) -> &mut Self {
		self.line_width = line_width;
		self
	}
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum DebugShapeVariant {
	Line { end: Point3 },
	Box { half_extents: Vec3 },
	Sphere { radius: f32 },
	Capsule { half_height: f32, radius: f32 },
	Plane { normal: Vec3, size: f32 },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DebugManager {
	#[serde(skip)]
	pub(crate) shapes: Vec<DebugShape>,

	#[serde(skip)]
	// FIXME: We need to hold on a reference for now due to resource manager collecting instantly
	#[allow(dead_code)]
	pub(crate) pipeline: Handle<GraphicsPipeline>,
}

impl DebugManager {
	const DEFAULT_COLOR: Color = Color::CYAN;
	const DEFAULT_LINE_WIDTH: f32 = 0.01;

	pub fn new() -> Self {
		Self {
			shapes: Vec::with_capacity(2048),
			pipeline: Handle::find_or_load("{063952B6-40B8-4D22-A26F-339185008B76}").unwrap(),
		}
	}

	pub fn draw_line(
		&mut self,
		a: impl Into<Point3>,
		b: impl Into<Point3>,
		life_time: f32,
	) -> &mut DebugShape {
		self.shapes.push(DebugShape {
			line_width: Self::DEFAULT_LINE_WIDTH,
			color: Self::DEFAULT_COLOR,

			time_left: life_time,

			location: a.into(),
			rotation: Quat::IDENTITY,
			variant: DebugShapeVariant::Line { end: b.into() },
		});
		self.shapes.last_mut().unwrap()
	}

	pub fn draw_box(
		&mut self,
		location: impl Into<Point3>,
		rotation: Quat,
		half_extents: impl Into<Vec3>,
		life_time: f32,
	) -> &mut DebugShape {
		self.shapes.push(DebugShape {
			line_width: Self::DEFAULT_LINE_WIDTH,
			color: Self::DEFAULT_COLOR,

			time_left: life_time,

			location: location.into(),
			rotation,
			variant: DebugShapeVariant::Box {
				half_extents: half_extents.into(),
			},
		});
		self.shapes.last_mut().unwrap()
	}

	pub fn draw_capsule(
		&mut self,
		location: Point3,
		rotation: Quat,
		half_height: f32,
		radius: f32,
		life_time: f32,
	) -> &mut DebugShape {
		self.shapes.push(DebugShape {
			line_width: Self::DEFAULT_LINE_WIDTH,
			color: Self::DEFAULT_COLOR,

			time_left: life_time,

			location,
			rotation,
			variant: DebugShapeVariant::Capsule {
				half_height,
				radius,
			},
		});
		self.shapes.last_mut().unwrap()
	}
}

impl Component for DebugManager {}

impl Default for DebugManager {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone)]
pub struct DebugSystem;
impl System for DebugSystem {
	fn run(&self, world: &World, dt: f32) {
		let mut debug_managers = world.write::<DebugManager>();
		let mut debug_manager = debug_managers.get_mut_or_default(world.singleton);

		// Update "time_left" for all shapes and then remove any that are "dead"
		debug_manager
			.shapes
			.iter_mut()
			.for_each(|e| e.time_left -= dt);
		debug_manager.shapes.retain(|e| e.time_left > 0.0)
	}
}
