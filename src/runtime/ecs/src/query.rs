use crate::ComponentVariantId;
use crate::Entity;
use crate::ReadStorage;
use crate::WriteStorage;
use crate::{Component, World};

use std::collections::HashSet;

#[derive(Default, Clone)]
pub struct Query {
	components: HashSet<ComponentVariantId>,
}

impl Query {
	pub fn new() -> Self {
		Self {
			components: HashSet::new(),
		}
	}

	pub fn read<T: Component>(mut self, _: &ReadStorage<'_, T>) -> Self {
		self.components.insert(T::VARIANT_ID);
		self
	}

	pub fn write<T: Component>(mut self, _: &WriteStorage<'_, T>) -> Self {
		self.components.insert(T::VARIANT_ID);
		self
	}

	pub async fn execute(self, world: &World) -> Vec<Entity> {
		let found = {
			let entities = world.entities.lock().await;
			entities
				.iter()
				.filter(|(_, info)| {
					!self
						.components
						.iter()
						.any(|c| info.components.get(c).is_none())
				})
				.map(|(id, info)| Entity {
					id: *id,
					info: info.clone(),
				})
				.collect()
		};

		found
	}
}
