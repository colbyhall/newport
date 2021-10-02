use super::Scene;
use super::SceneRuntime;
use super::{
	Component,
	ComponentsContainer,

	Entity,
	EntityInfo,
};
use asset::AssetRef;
use engine::Engine;

use std::sync::RwLock;

#[derive(Default)]
pub struct World {
	pub(crate) persistent_scene: RwLock<SceneRuntime>,
	pub(crate) components: ComponentsContainer,
}

impl Clone for World {
	fn clone(&self) -> Self {
		let read = self.persistent_scene.read().unwrap();

		Self {
			persistent_scene: RwLock::new((*read).clone()),
			components: self.components.clone(),
		}
	}
}

impl World {
	pub fn new(persistent_scene: &AssetRef<Scene>) -> Self {
		let result = Self {
			persistent_scene: Default::default(),
			components: ComponentsContainer::new(Engine::register().unwrap().clone()),
		};

		{
			let mut scene = result.persistent_scene.write().unwrap();

			for it in persistent_scene.entities.iter() {
				let components = it
					.components
					.iter()
					.map(|(key, value)| {
						let mut write = result.components.write_id(*key).unwrap();
						(*key, write.insert_box(value))
					})
					.collect();

				scene.entities.insert(it.id, EntityInfo { components });
			}
		}

		result
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
		let mut scene = self.world.persistent_scene.write().unwrap();
		let id = Entity::new();
		scene.entities.insert(id, self.entity_info);
		id
	}
}
