use std::collections::HashMap;

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
	pub(crate) entities: Mutex<EntityContainer>,
	pub singleton: Entity,
}

impl World {
	pub fn new() -> Self {
		let singleton = Entity::new();

		let variants: &[ComponentVariant] = Engine::register();
		let variants = variants.iter().map(|v| (v.id, v.clone())).collect();

		let world = Self {
			components: ComponentsContainer::new(),
			variants,
			entities: Mutex::new(EntityContainer::with_capacity(2048 * 8)),
			singleton,
		};

		{
			let mut entities = world.entities.lock().unwrap();
			entities.insert(singleton, EntityInfo::default());
		}

		world
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
		self.components.write(self)
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

		T::on_added(self, entity, storage);
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
}

impl Default for World {
	fn default() -> Self {
		Self::new()
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
		T::on_added(self.world, self.entity, storage);

		self
	}

	pub fn finish(self) -> Entity {
		self.entity
	}
}
