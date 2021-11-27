use std::collections::HashMap;

use super::ComponentVariantId;
use super::Entity;
use super::ReadStorage;
use super::Scene;
use super::SceneRuntime;
use super::WriteStorage;
// use super::physics::PhysicsWorld;
use super::{
	Component,
	ComponentsContainer,

	EntityId,
	EntityInfo,
};
use engine::Engine;
use resources::Handle;
use std::any::Any;
use sync::lock::Mutex;

pub struct World {
	pub(crate) persistent_scene: Mutex<SceneRuntime>,
	pub(crate) components: ComponentsContainer,
	// pub(crate) physics: Mutex<PhysicsWorld>,
}

impl World {
	pub fn new(persistent_scene: &Handle<Scene>) -> Self {
		let result = Self {
			persistent_scene: Default::default(),
			components: ComponentsContainer::new(Engine::register().unwrap().clone()),
			// physics: Mutex::new(PhysicsWorld::new())
		};

		// let mut scene = result.persistent_scene.get_mut();

		// for it in persistent_scene.entities.iter() {
		// 	let components = it
		// 		.components
		// 		.iter()
		// 		.map(|(key, value)| {
		// 			let mut write = result.components.write_id(*key).unwrap();
		// 			(*key, write.insert_box(value))
		// 		})
		// 		.collect();

		// 	scene.entities.insert(it.id, EntityInfo { components });
		// }

		result
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
		let scene = self.persistent_scene.lock().await;
		let info = scene.entities.get(&id)?.clone();
		Some(Entity { id, info })
	}
}

impl Default for World {
	fn default() -> Self {
		Self {
			persistent_scene: Default::default(),
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

		let mut scene = world.persistent_scene.lock().await;
		let id = EntityId::new();

		let mut entity_info = EntityInfo::default();

		for (variant, component) in components.iter() {
			let mut write = self.world.components.write_id(*variant).await;
			let id = write.insert_box(component);
			entity_info.components.insert(*variant, id);
		}

		scene.entities.insert(id, entity_info);

		// let entity = world.find(id).await.unwrap();
		// for c in entity.info.components.keys() {
		// 	let variant = world.components.variants.get(c).unwrap();
		// 	if let Some(on_added) = variant.on_added {
		// 		on_added(world, &entity).await;
		// 	}
		// }

		id
	}
}
