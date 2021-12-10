use std::collections::HashMap;

use crate::{
	Component,
	ComponentVariantId,
	ComponentsContainer,
	Entity,
	EntityContainer,
	EntityInfo,
	ReadStorage,
	WriteStorage,
};
use engine::Engine;
use std::any::Any;
use sync::lock::Mutex;

pub struct World {
	pub(crate) entities: Mutex<EntityContainer>,
	pub(crate) components: ComponentsContainer,
}

impl World {
	pub fn new() -> Self {
		Self {
			entities: Mutex::new(EntityContainer::new()),
			components: ComponentsContainer::new(Engine::register().to_vec()),
		}
	}

	pub fn spawn(&self) -> EntityBuilder<'_> {
		EntityBuilder {
			world: self,
			components: HashMap::with_capacity(32),
		}
	}

	pub async fn read<T: Component>(&self) -> ReadStorage<'_, T> {
		self.components.read().await
	}

	pub async fn write<T: Component>(&self) -> WriteStorage<'_, T> {
		self.components.write().await
	}
}

impl Default for World {
	fn default() -> Self {
		Self {
			entities: Default::default(),
			components: ComponentsContainer::new(Engine::register().to_vec()),
		}
	}
}

pub struct EntityBuilder<'a> {
	world: &'a World,
	components: HashMap<ComponentVariantId, Box<dyn Any>>, // TODO: Use temp allocator here
}

impl<'a> EntityBuilder<'a> {
	pub fn with<T: Component>(mut self, t: T) -> Self {
		self.components.insert(T::VARIANT_ID, Box::new(t));
		self
	}

	pub async fn finish(self) -> Entity {
		let EntityBuilder { world, components } = self;

		let mut entities = world.entities.lock().await;
		let id = Entity::new();

		let mut entity_info = EntityInfo::default();
		for (variant, component) in components.iter() {
			let mut write = self.world.components.write_id(*variant).await;
			write.insert_box(id, component);
			entity_info.components |= variant.to_mask();
		}
		entities.insert(id, entity_info);

		id
	}
}
