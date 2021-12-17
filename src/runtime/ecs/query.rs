use crate::{
	Component,
	Entity,
	EntityInfo,
	ReadStorage,
	World,
	WriteStorage,
};

#[derive(Default, Clone)]
pub struct Query {
	info: EntityInfo,
}

impl Query {
	pub fn new() -> Self {
		Self {
			info: EntityInfo::default(),
		}
	}

	pub fn read<T: Component>(mut self, _: &ReadStorage<'_, T>) -> Self {
		self.info.components |= T::VARIANT_ID.to_mask();
		self
	}

	pub fn write<T: Component>(mut self, _: &WriteStorage<'_, T>) -> Self {
		self.info.components |= T::VARIANT_ID.to_mask();
		self
	}

	pub async fn execute(self, world: &World) -> Vec<Entity> {
		let found = {
			let entities = world.entities.lock().await;
			entities
				.iter()
				.filter(|(_, info)| {
					(info.components & self.info.components) == self.info.components
				})
				.map(|(id, _)| *id)
				.collect()
		};

		found
	}
}
