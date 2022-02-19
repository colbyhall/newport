use std::collections::HashMap;

use engine::Uuid;
use resources::Handle;

use crate::{
	Scene,
	SceneCollection,
	SceneRuntime,
};

use {
	crate::{
		Component,
		ComponentId,
		ComponentVariant,
		ComponentsContainer,
		Entity,
		EntityContainer,
		EntityInfo,
		ReadStorage,
		WriteStorage,
	},
	engine::Engine,
	std::sync::{
		Mutex,
		MutexGuard,
	},
};

pub struct World {
	pub variants: HashMap<ComponentId, ComponentVariant>,
	pub(crate) components: ComponentsContainer,
	pub(crate) scenes: Mutex<SceneCollection>,
	pub singleton: Entity,
	pub persistent: Uuid,
}

impl World {
	pub fn new(persistent: Option<&Handle<Scene>>) -> Self {
		let persistent_id = persistent.map(|f| f.uuid()).unwrap_or_default();
		let singleton = Entity::new(persistent_id);

		let variants: &[ComponentVariant] = Engine::register();
		let variants = variants.iter().map(|v| (v.id, v.clone())).collect();

		let world = Self {
			components: ComponentsContainer::new(),
			variants,
			scenes: Mutex::new(SceneCollection::with_capacity(32)),
			singleton,
			persistent: persistent_id,
		};

		// Add the first initial scene
		if let Some(scene) = persistent {
			world.push_scene(scene);

			// TODO: This is a double lock maybe i should make this better
			let mut scenes = world.scenes.lock().unwrap();
			let persistent = scenes.get_mut(&world.persistent).unwrap();
			persistent.entities.insert(singleton, EntityInfo::default());
		} else {
			let mut scenes = world.scenes.lock().unwrap();

			let mut entities = EntityContainer::new();
			entities.insert(singleton, EntityInfo::default());
			scenes.insert(
				persistent_id,
				SceneRuntime {
					scene: None,
					entities,
				},
			);
		}

		world
	}

	pub fn push_scene(&self, scene: &Handle<Scene>) {
		let mut scenes = self.scenes.lock().unwrap();

		let mut entities = EntityContainer::new();
		{
			let actual = scene.read();
			for e in actual.entities.iter() {
				let entity = Entity {
					id: e.id,
					scene: scene.uuid(),
				};

				let mut info = EntityInfo::default();
				for (id, c) in e.components.iter() {
					let mut storage = self.components.write_id(*id);
					let mask = id.to_mask();
					info.components |= mask;
					storage.insert_box(entity, c);

					// TODO: PostLoad????
					// Call the on added method
					// let variant = self.variants.get(&T::VARIANT_ID).unwrap();
					// if let Some(on_added) = variant.on_added {
					// 	(on_added)(entity, &mut storage.storage);
					// }
				}
				entities.insert(entity, info);
			}
		}

		scenes.insert(
			scene.uuid(),
			SceneRuntime {
				scene: Some(scene.clone()),
				entities,
			},
		);
	}

	pub fn spawn(&self, scene_id: Uuid) -> EntityBuilder<'_> {
		let mut scenes = self.scenes.lock().unwrap();

		let scene = match scenes.get_mut(&scene_id) {
			Some(x) => x,
			None => scenes.get_mut(&self.persistent).unwrap(),
		};

		let entity = Entity::new(scene_id);
		scene.entities.insert(entity, EntityInfo::default());
		EntityBuilder {
			world: self,
			scenes,
			entity,
		}
	}

	pub fn read<T: Component>(&self) -> ReadStorage<'_, T> {
		self.components.read()
	}

	pub fn write<T: Component>(&self) -> WriteStorage<'_, T> {
		self.components.write(self)
	}

	pub fn insert<T: Component>(&self, storage: &mut WriteStorage<'_, T>, entity: Entity, t: T) {
		let mut scenes = self.scenes.lock().unwrap();
		let scene = scenes.get_mut(&entity.scene).unwrap();
		let info = scene.entities.get_mut(&entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			storage.storage.remove(entity);
		}
		info.components |= mask;
		storage.storage.insert(entity, t);

		T::on_added(self, entity, storage);
	}

	pub fn remove<T: Component>(&self, storage: &mut WriteStorage<'_, T>, entity: Entity) -> bool {
		let mut scenes = self.scenes.lock().unwrap();
		let scene = scenes.get_mut(&entity.scene).unwrap();
		let info = scene.entities.get_mut(&entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			info.components &= !mask;
			storage.storage.remove(entity)
		} else {
			false
		}
	}
}

impl Default for World {
	fn default() -> Self {
		Self::new(None)
	}
}

pub struct EntityBuilder<'a> {
	world: &'a World,
	scenes: MutexGuard<'a, SceneCollection>,
	entity: Entity,
}

impl<'a> EntityBuilder<'a> {
	#[must_use]
	pub fn with<T: Component>(mut self, t: T, storage: &mut WriteStorage<T>) -> Self {
		let scene = self.scenes.get_mut(&self.entity.scene).unwrap();
		let info = scene.entities.get_mut(&self.entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			info.components &= !mask;
			storage.storage.remove(self.entity);
		}

		info.components |= mask;
		storage.storage.insert(self.entity, t);

		// Call the on added method
		T::on_added(self.world, self.entity, storage);

		self
	}

	pub fn finish(self) -> Entity {
		self.entity
	}
}
