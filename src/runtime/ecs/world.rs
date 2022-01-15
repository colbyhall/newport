use std::collections::HashMap;

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
	pub(crate) entities: Mutex<EntityContainer>,
	pub(crate) components: ComponentsContainer,
	pub singleton: Entity,
	schedule: ScheduleBlock,
	pub variants: HashMap<ComponentVariantId, ComponentVariant>,
}

impl World {
	pub fn new(schedule: ScheduleBlock) -> Self {
		let mut entities = EntityContainer::new();

		let singleton = Entity::new();
		entities.insert(singleton, EntityInfo::default());

		let variants: &[ComponentVariant] = Engine::register();
		let variants = variants.iter().map(|v| (v.id, v.clone())).collect();

		Self {
			entities: Mutex::new(entities),
			components: ComponentsContainer::new(),
			singleton,
			schedule,
			variants,
		}
	}

	pub fn spawn(&self) -> EntityBuilder<'_> {
		let mut entities = self.entities.lock().unwrap();
		let entity = Entity::new();
		entities.insert(entity, EntityInfo::default());
		EntityBuilder {
			world: self,
			entities,
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
		let mut entities = self.entities.lock().unwrap();
		let info = entities.get_mut(&entity).unwrap();

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
		let mut entities = self.entities.lock().unwrap();
		let info = entities.get_mut(&entity).unwrap();

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
		Self::new(ScheduleBlock::default())
	}
}

pub struct EntityBuilder<'a> {
	world: &'a World,
	entities: MutexGuard<'a, EntityContainer>,
	entity: Entity,
}

impl<'a> EntityBuilder<'a> {
	#[must_use]
	pub fn with<T: Component>(mut self, t: T, storage: &mut WriteStorage<T>) -> Self {
		let info = self.entities.get_mut(&self.entity).unwrap();

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
