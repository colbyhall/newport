use ecs::{
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

		Self {
			world: World::new(),
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
