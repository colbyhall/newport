use crate::{
	Component,
	ComponentsContainer,
	EntitiesContainer,

	Entity,
	EntityInfo,
};
use engine::Engine;
use serde::{
	Deserialize,
	Serialize,
};
use std::sync::RwLock;

#[derive(Default, Serialize, Deserialize)]
pub struct World {
	pub(crate) entities: RwLock<EntitiesContainer>,
	pub(crate) components: ComponentsContainer,
}

impl Clone for World {
	fn clone(&self) -> Self {
		let read = self.entities.read().unwrap();

		Self {
			entities: RwLock::new((*read).clone()),
			components: self.components.clone(),
		}
	}
}

impl asset::Asset for World {}

impl World {
	pub fn new() -> Self {
		let component_variants = Engine::register();
		Self {
			entities: RwLock::new(EntitiesContainer::new()),
			components: ComponentsContainer::new(component_variants),
		}
	}

	pub fn create(&self) -> EntityBuilder<'_> {
		EntityBuilder {
			world: self,
			entity_info: EntityInfo::default(),
		}
	}
}

pub struct EntityBuilder<'a> {
	world: &'a World,
	entity_info: EntityInfo,
}

impl<'a> EntityBuilder<'a> {
	pub fn with<T: Component>(mut self, t: T) -> Self {
		let mut write = self.world.components.write::<T>().unwrap_or_else(|| {
			panic!(
				"Component type \"{}\" not registered.",
				std::any::type_name::<T>()
			)
		});
		self.entity_info
			.components
			.insert(T::VARIANT_ID, write.insert(t));
		self
	}

	pub fn spawn(self) -> Entity {
		let mut entities = self.world.entities.write().unwrap();
		entities.insert(self.entity_info)
	}
}
