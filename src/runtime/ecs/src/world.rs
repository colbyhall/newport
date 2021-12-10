use {
	crate::{
		Component,
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
	std::{
		any::Any,
		collections::HashMap,
	},
	sync::lock::Mutex,
};

pub struct World {
	pub(crate) entities: Mutex<EntityContainer>,
	pub(crate) components: ComponentsContainer,
	pub singleton: Entity,
	schedule: ScheduleBlock,
}

impl World {
	pub fn new(schedule: ScheduleBlock) -> Self {
		let mut entities = EntityContainer::new();

		let singleton = Entity::new();
		entities.insert(singleton, EntityInfo::default());

		Self {
			entities: Mutex::new(entities),
			components: ComponentsContainer::new(Engine::register().to_vec()),
			singleton,
			schedule,
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

	pub async fn insert<T: Component>(
		&self,
		storage: &mut WriteStorage<'_, T>,
		entity: Entity,
		t: T,
	) {
		let mut entities = self.entities.lock().await;
		let info = entities.get_mut(&entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			storage.storage.remove(entity);
		}
		info.components |= mask;
		storage.storage.insert(entity, t);
	}

	pub async fn remove<T: Component>(
		&self,
		storage: &mut WriteStorage<'_, T>,
		entity: Entity,
	) -> bool {
		let mut entities = self.entities.lock().await;
		let info = entities.get_mut(&entity).unwrap();

		let mask = T::VARIANT_ID.to_mask();
		if info.components & mask == mask {
			info.components &= !mask;
			storage.storage.remove(entity)
		} else {
			false
		}
	}

	pub async fn step(&'static self, dt: f32) {
		self.schedule.execute(self, dt).await;
	}
}

impl Default for World {
	fn default() -> Self {
		Self::new(ScheduleBlock::default())
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
