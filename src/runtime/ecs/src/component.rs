use serde::de::DeserializeOwned;
use serde::{
	self,
	ron,
	Deserialize,
	Serialize,
};

use std::collections::HashMap;
use std::marker::PhantomData;
use std::{
	any::{
		type_name,
		Any,
	},
	collections::VecDeque,
};

use sync::lock::{
	Mutex,
	MutexGuard,
};

use crate::{
	Entity,
	EntityInfo,
};

pub trait Component:
	Sync + Send + Sized + Clone + Serialize + DeserializeOwned + Default + 'static
{
	const VARIANT_ID: ComponentVariantId = ComponentVariantId::new(type_name::<Self>());

	fn variant() -> ComponentVariant {
		fn create_storage<T: Component>() -> Box<dyn DynamicStorage> {
			Box::new(Storage::<T>::new())
		}

		fn parse_value<T: Component>(value: ron::Value) -> ron::Result<Box<dyn Any>> {
			let t: T = value.into_rust()?;
			Ok(Box::new(t))
		}

		ComponentVariant {
			name: type_name::<Self>()
				.rsplit_once("::")
				.unwrap_or(("", type_name::<Self>()))
				.1,
			id: Self::VARIANT_ID,

			create_storage: create_storage::<Self>,
			parse_value: parse_value::<Self>,
		}
	}
}

impl<T> Component for T
where
	T: Sync + Send + Sized + Clone + Serialize + DeserializeOwned + Default + 'static,
{
	default const VARIANT_ID: ComponentVariantId = ComponentVariantId::new(type_name::<Self>());
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ComponentVariantId(u32);

impl ComponentVariantId {
	pub const fn new(name: &'static str) -> Self {
		const FNV_OFFSET_BASIC: u64 = 2166136261;
		// const FNV_PRIME: u64 = 16777619;

		const fn hash_rec(name: &'static str, index: usize, hash: u64) -> u64 {
			let hash = hash ^ name.as_bytes()[index] as u64;
			if index != name.len() - 1 {
				hash_rec(name, index + 1, hash)
			} else {
				hash
			}
		}

		Self(hash_rec(name, 0, FNV_OFFSET_BASIC) as u32)
	}

	pub const fn to_mask(self) -> u128 {
		1 << (self.0 as usize & (EntityInfo::MAX_COMPONENT_TYPES - 1)) as u128
	}
}

#[derive(Clone)]
pub struct ComponentVariant {
	pub name: &'static str,

	pub id: ComponentVariantId,

	create_storage: fn() -> Box<dyn DynamicStorage>,
	pub parse_value: fn(value: ron::Value) -> ron::Result<Box<dyn Any>>,
}

pub(crate) trait DynamicStorage: Send + Sync + DynamicStorageClone + 'static {
	fn insert_box(&mut self, entity: Entity, value: &Box<dyn Any>);
	fn remove(&mut self, entity: Entity) -> bool;

	fn as_any_mut(&mut self) -> &mut dyn Any;
	fn as_any(&self) -> &dyn Any;
}

pub(crate) trait DynamicStorageClone {
	fn clone_to_box(&self) -> Box<dyn DynamicStorage>;
}

impl<T: DynamicStorage + Clone> DynamicStorageClone for T {
	fn clone_to_box(&self) -> Box<dyn DynamicStorage> {
		Box::new(self.clone())
	}
}

#[derive(Clone)]
struct Storage<T: Component> {
	// SPEED: We're also going to need to keep these somehow organized for best iteration order
	components: Vec<Option<T>>,
	available: VecDeque<usize>,

	entity_to_index: HashMap<Entity, usize>,
}

impl<T: Component> Storage<T> {
	fn new() -> Self {
		let capacity = 512;
		Self {
			components: Vec::with_capacity(capacity),
			available: VecDeque::with_capacity(64),
			entity_to_index: HashMap::with_capacity(capacity),
		}
	}

	fn insert(&mut self, entity: Entity, t: T) {
		let index = if self.available.is_empty() {
			let index = self.components.len();
			self.components.push(Some(t));
			index
		} else {
			let index = self.available.pop_front().unwrap();
			self.components[index] = Some(t);
			index
		};
		self.entity_to_index.insert(entity, index);
	}

	fn get(&self, entity: Entity) -> Option<&T> {
		let index = self
			.entity_to_index
			.get(&entity)
			.cloned()
			.unwrap_or(self.components.len());

		if self.components.len() <= index {
			return None;
		}

		self.components[index].as_ref()
	}

	fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
		let index = self
			.entity_to_index
			.get(&entity)
			.cloned()
			.unwrap_or(self.components.len());

		if self.components.len() <= index {
			return None;
		}

		self.components[index].as_mut()
	}
}

impl<T: Component> DynamicStorage for Storage<T> {
	fn remove(&mut self, entity: Entity) -> bool {
		let index = self
			.entity_to_index
			.get(&entity)
			.cloned()
			.unwrap_or(self.components.len());

		if self.components.len() <= index {
			return false;
		}

		self.available.push_back(index);
		self.components[index].take().is_some()
	}

	fn insert_box(&mut self, entity: Entity, value: &Box<dyn Any>) {
		let t: &T = value.downcast_ref().unwrap();
		self.insert(entity, t.clone())
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

#[derive(Default)]
pub struct ComponentsContainer {
	// TODO: This one day needs to use an async rwlock
	map: HashMap<ComponentVariantId, Mutex<Box<dyn DynamicStorage>>>,
	pub variants: HashMap<ComponentVariantId, ComponentVariant>,
}

impl ComponentsContainer {
	pub fn new(mut variants: Vec<ComponentVariant>) -> Self {
		let map = variants
			.iter()
			.map(|v| (v.id, Mutex::new((v.create_storage)())))
			.collect();
		let variants = variants.drain(..).map(|v| (v.id, v)).collect();
		Self { map, variants }
	}

	pub async fn read_id(&self, variant_id: ComponentVariantId) -> AnyReadStorage<'_> {
		AnyReadStorage {
			read: self
				.map
				.get(&variant_id)
				.expect("Unregistered component type")
				.lock()
				.await,
		}
	}

	pub async fn read<T: Component>(&self) -> ReadStorage<'_, T> {
		ReadStorage {
			storage: self.read_id(T::VARIANT_ID).await,
			phantom: PhantomData,
		}
	}

	pub async fn write_id(&self, variant_id: ComponentVariantId) -> AnyWriteStorage<'_> {
		AnyWriteStorage {
			write: self
				.map
				.get(&variant_id)
				.expect("Unregistered component type")
				.lock()
				.await,
		}
	}

	pub async fn write<T: Component>(&self) -> WriteStorage<'_, T> {
		WriteStorage {
			storage: self.write_id(T::VARIANT_ID).await,
			phantom: PhantomData,
		}
	}
}

pub struct AnyReadStorage<'a> {
	read: MutexGuard<'a, Box<dyn DynamicStorage>>,
}

impl<'a> AnyReadStorage<'a> {
	pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
		self.read
			.as_any()
			.downcast_ref::<Storage<T>>()
			.expect("Incorrect usage of ReadStorage")
			.get(entity)
	}
}

pub struct ReadStorage<'a, T: Component> {
	storage: AnyReadStorage<'a>,
	phantom: PhantomData<T>,
}

impl<'a, T: Component> ReadStorage<'a, T> {
	pub fn get(&self, entity: &Entity) -> Option<&T> {
		self.storage.get(*entity)
	}
}

pub struct AnyWriteStorage<'a> {
	write: MutexGuard<'a, Box<dyn DynamicStorage>>,
}

impl<'a> AnyWriteStorage<'a> {
	pub fn insert<T: Component>(&mut self, entity: Entity, t: T) {
		self.write
			.as_any_mut()
			.downcast_mut::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.insert(entity, t)
	}

	pub fn insert_box(&mut self, entity: Entity, value: &Box<dyn Any>) {
		self.write.insert_box(entity, value)
	}

	pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
		self.write
			.as_any()
			.downcast_ref::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.get(entity)
	}

	pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
		self.write
			.as_any_mut()
			.downcast_mut::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.get_mut(entity)
	}

	pub fn remove(&mut self, entity: Entity) -> bool {
		self.write.remove(entity)
	}
}

pub struct WriteStorage<'a, T: Component> {
	pub(crate) storage: AnyWriteStorage<'a>,
	phantom: PhantomData<T>,
}

impl<'a, T: Component> WriteStorage<'a, T> {
	pub fn get(&self, entity: &Entity) -> Option<&T> {
		self.storage.get(*entity)
	}

	pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
		self.storage.get_mut(*entity)
	}
}
