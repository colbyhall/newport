use crate::{
	Component,
	ComponentVariant,
	ComponentsContainer,
	EntitiesContainer,

	Entity,
	EntityInfo,
};

use std::sync::RwLock;

#[derive(Default)]
pub struct World {
	pub(crate) entities: RwLock<EntitiesContainer>,
	pub(crate) components: ComponentsContainer,
}

impl World {
	pub fn new(component_variants: Vec<ComponentVariant>) -> Self {
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
		let mut write = self.world.components.write::<T>().unwrap();
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
