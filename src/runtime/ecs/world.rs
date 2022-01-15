use std::collections::HashMap;

use resources::Handle;

use crate::{
	Scene,
	SceneCollection,
	SceneId,
	SceneRuntime,
};

use {
	crate::{
		Component,
		ComponentVariant,
		ComponentVariantId,
		ComponentsContainer,
		Entity,
		EntityContainer,
		EntityInfo,
		ReadStorage,
		ScheduleBlock,
		WriteStorage,
	},
	engine::Engine,
	std::sync::{
		Mutex,
		MutexGuard,
	},
};

pub struct World {
	pub(crate) components: ComponentsContainer,
	pub singleton: Entity,
	schedule: ScheduleBlock,
	pub variants: HashMap<ComponentVariantId, ComponentVariant>,
	pub(crate) scenes: Mutex<SceneCollection>,
}

impl World {
	pub fn new(persistent: Option<&Handle<Scene>>, schedule: ScheduleBlock) -> Self {
		let singleton = Entity::new(SceneId::PERSISTENT);
		// entities.insert(singleton, EntityInfo::default());

		let variants: &[ComponentVariant] = Engine::register();
		let variants = variants.iter().map(|v| (v.id, v.clone())).collect();

		let world = Self {
			components: ComponentsContainer::new(),
			singleton,
			schedule,
			variants,
			scenes: Mutex::new(SceneCollection::with_capacity(32)),
		};

		// Add the first initial scene
		if let Some(scene) = persistent {
			world.push_scene(scene);

			// TODO: This is a double lock maybe i should make this better
			let mut scenes = world.scenes.lock().unwrap();
			let persistent = scenes[SceneId::PERSISTENT.0].as_mut().unwrap();
			persistent.entities.insert(singleton, EntityInfo::default());
		} else {
			let mut scenes = world.scenes.lock().unwrap();

			let mut entities = EntityContainer::new();
			entities.insert(singleton, EntityInfo::default());
			scenes.push(Some(SceneRuntime {
				scene: None,
				entities,
			}))
		}

		world
	}

	pub fn push_scene(&self, scene: &Handle<Scene>) -> Option<SceneId> {
		let mut scenes = self.scenes.lock().unwrap();

		// Make sure we havent added this scene before and also cache an empty spot if we could find one
		let mut result = None;
		for (index, it) in scenes.iter().enumerate() {
			if let Some(it) = it {
				if it.scene.as_ref() == Some(scene) {
					return None;
				}
			} else {
				result = Some(index);
			}
		}
		let result = SceneId(result.unwrap_or_else(|| scenes.len()));

		let mut entities = EntityContainer::new();
		{
			let scene = scene.read();
			for e in scene.entities.iter() {
				let entity = Entity {
					id: e.id,
					scene: result,
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

		let runtime = SceneRuntime {
			scene: Some(scene.clone()),
			entities,
		};
		if let Some(at) = scenes.get_mut(result.0) {
			*at = Some(runtime);
		} else {
			scenes.push(Some(runtime));
		}

		Some(result)
	}

	pub fn spawn(&self, scene: SceneId) -> EntityBuilder<'_> {
		let mut scenes = self.scenes.lock().unwrap();
		// TODO: Should we return an optional here?
		// If we can't find the specificed scene just throw it into the persistent scene
		let (id, scene) = match scenes.get_mut(scene.0) {
			Some(Some(x)) => (scene, x),
			_ => (
				SceneId::PERSISTENT,
				scenes[SceneId::PERSISTENT.0].as_mut().unwrap(),
			),
		};

		let entity = Entity::new(id);
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
		self.components.write()
	}

	pub fn insert<T: Component>(&self, storage: &mut WriteStorage<'_, T>, entity: Entity, t: T) {
		let mut scenes = self.scenes.lock().unwrap();
		let scene = scenes[entity.scene.0].as_mut().unwrap();
		let info = scene.entities.get_mut(&entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			storage.storage.remove(entity);
		}
		info.components |= mask;
		storage.storage.insert(entity, t);

		// Call the on added method
		let variant = self.variants.get(&T::VARIANT_ID).unwrap();
		if let Some(on_added) = variant.on_added {
			(on_added)(entity, &mut storage.storage);
		}
	}

	pub fn remove<T: Component>(&self, storage: &mut WriteStorage<'_, T>, entity: Entity) -> bool {
		let mut scenes = self.scenes.lock().unwrap();
		let scene = scenes[entity.scene.0].as_mut().unwrap();
		let info = scene.entities.get_mut(&entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			info.components &= !mask;
			storage.storage.remove(entity)
		} else {
			false
		}
	}

	pub fn step(&'static self, dt: f32) {
		self.schedule.execute(self, dt);
	}
}

impl Default for World {
	fn default() -> Self {
		Self::new(None, ScheduleBlock::default())
	}
}

pub struct EntityBuilder<'a> {
	world: &'a World,
	scenes: MutexGuard<'a, Vec<Option<SceneRuntime>>>,
	entity: Entity,
}

impl<'a> EntityBuilder<'a> {
	#[must_use]
	pub fn with<T: Component>(mut self, t: T, storage: &mut WriteStorage<T>) -> Self {
		let scene = self.scenes[self.entity.scene.0].as_mut().unwrap();
		let info = scene.entities.get_mut(&self.entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			info.components &= !mask;
			storage.storage.remove(self.entity);
		}

		info.components |= mask;
		storage.storage.insert(self.entity, t);

		// Call the on added method
		let variant = self.world.variants.get(&T::VARIANT_ID).unwrap();
		if let Some(on_added) = variant.on_added {
			(on_added)(self.entity, &mut storage.storage);
		}

		self
	}

	pub fn finish(self) -> Entity {
		self.entity
	}
}
