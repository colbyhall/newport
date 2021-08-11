  
use math::Matrix4;

use crate::{
	RenderState,
};

use {
	ecs::World,
	engine::Engine,
};

use std::collections::HashMap;

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub struct ViewportId(u64);

#[derive(Clone)]
pub struct Viewport {
	pub width: u32,
	pub height: u32,

	pub transform: Matrix4,
	pub fov: f32,
}

#[derive(Default)]
pub struct GameState {
	world: World,

	last_viewport_id: u64,
	viewports: HashMap<ViewportId, Viewport>,
}

impl GameState {
	pub fn new() -> Self {
		let systems = Engine::as_ref().register().unwrap_or_default();
		let world = World::new(systems);

		// world
		// 	.create()
		// 	.with(Named {
		// 		name: "Hello World".into(),
		// 	})
		// 	.finish();

		// world.create().with(Named { name: "Foo".into() }).finish();

		// world.create().with(Named { name: "Bar".into() }).finish();

		// world.create().with(Named { name: "Car".into() }).finish();

		Self {
			world,

			last_viewport_id: 0,
			viewports: HashMap::new(),
		}
	}

	pub fn simulate(&mut self, _dt: f32) -> RenderState {
		// self.world.simulate(dt);

		RenderState {
			viewports: self.viewports.clone(),

			primitives: Vec::new(),
			primitive_transforms: Vec::new(),
		}
	}

	pub fn register_viewport(&mut self, viewport: Viewport) -> ViewportId {
		let id = ViewportId(self.last_viewport_id);
		self.last_viewport_id += 1;
		self.viewports.insert(id, viewport);
		id
	}

	pub fn unregister_viewport(&mut self, id: ViewportId) {
		self.viewports.remove(&id);
	}

	pub fn viewport(&self, id: ViewportId) -> Option<&Viewport> {
		self.viewports.get(&id)
	}

	pub fn viewport_mut(&mut self, id: ViewportId) -> Option<&mut Viewport> {
		self.viewports.get_mut(&id)
	}

	pub fn world(&self) -> &World {
		&self.world
	}
}