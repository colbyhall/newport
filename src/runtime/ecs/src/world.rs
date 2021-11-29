use std::collections::HashMap;

use crate::ComponentVariantId;
use crate::Entity;
use crate::EntityContainer;
use crate::ReadStorage;
use crate::WriteStorage;
// use super::physics::PhysicsWorld;
use crate::{Component, ComponentsContainer, EntityId, EntityInfo};
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
			components: ComponentsContainer::new(Engine::register().unwrap().clone()),
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

	pub async fn find(&self, id: EntityId) -> Option<Entity> {
		let entities = self.entities.lock().await;
		let info = entities.get(&id)?.clone();
		Some(Entity { id, info })
	}
}

impl Default for World {
	fn default() -> Self {
		Self {
			entities: Default::default(),
			components: ComponentsContainer::new(Engine::register().unwrap().clone()),
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

	pub async fn finish(self) -> EntityId {
		let EntityBuilder { world, components } = self;

		let mut entities = world.entities.lock().await;
		let id = EntityId::new();

		let mut entity_info = EntityInfo::default();
		for (variant, component) in components.iter() {
			let mut write = self.world.components.write_id(*variant).await;
			let id = write.insert_box(component);
			entity_info.components.insert(*variant, id);
		}
		entities.insert(id, entity_info);

		id
	}
}
