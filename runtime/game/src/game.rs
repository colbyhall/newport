use crate::ecs::{
	Schedule,
	World,
};
use asset::AssetRef;

use crate::systems::*;

#[derive(Default)]
pub struct GameState {
	world: World,
	schedule: Schedule,
}

impl GameState {
	pub fn new() -> Self {
		let schedule = Schedule::builder()
			.single(Box::new(SpinDriver))
			.single(Box::new(ScaleDriver))
			.single(Box::new(CameraDriver))
			.spawn();

		let default_scene = AssetRef::new("{CB80A291-A3D8-4D1A-A702-33EFBCA02DDE}").unwrap();

		Self {
			world: World::new(&default_scene),
			schedule,
		}
	}

	pub async fn simulate(&self, dt: f32) {
		let Self { world, schedule } = self;
		schedule.execute(world, dt).await
	}

	pub fn world(&self) -> &World {
		&self.world
	}
}
