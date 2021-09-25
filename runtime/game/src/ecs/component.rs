use serde::de::DeserializeOwned;
use serde::{
	self,
	ron,
	Deserialize,
	Serialize,
};

use std::collections::HashMap;
use std::{
	any::{
		type_name,
		Any,
	},
	collections::VecDeque,
	sync::{
		RwLock,
		RwLockReadGuard,
		RwLockWriteGuard,
	},
};

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
}

pub trait Component:
	Sync + Send + Sized + Clone + Serialize + DeserializeOwned + Default + 'static
{
	const VARIANT_ID: ComponentVariantId = ComponentVariantId::new(type_name::<Self>());
}

pub trait Singleton {}

impl<T> Component for T
where
	T: Sync + Send + Sized + Clone + Serialize + DeserializeOwned + Default + 'static,
{
	default const VARIANT_ID: ComponentVariantId = ComponentVariantId::new(type_name::<Self>());
}

#[derive(Clone)]
pub struct ComponentVariant {
	pub name: &'static str,

	pub id: ComponentVariantId,

	create_storage: fn() -> Box<dyn DynamicStorage>,
	pub parse_value: fn(value: ron::Value) -> ron::Result<Box<dyn Any>>,
}

impl ComponentVariant {
	pub fn new<T: Component>() -> Self {
		fn create_storage<T: Component>() -> Box<dyn DynamicStorage> {
			Box::new(Storage::<T>::new())
		}

		fn parse_value<T: Component>(value: ron::Value) -> ron::Result<Box<dyn Any>> {
			let t: T = value.into_rust()?;
			Ok(Box::new(t))
		}

		Self {
			name: type_name::<T>()
				.rsplit_once("::")
				.unwrap_or(("", type_name::<T>()))
				.1,
			id: T::VARIANT_ID,

			create_storage: create_storage::<T>,
			parse_value: parse_value::<T>,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
	variant_id: ComponentVariantId,

	index: u32,
	generation: u32,
}

#[derive(Clone)]
struct Storage<T: Component> {
	// SPEED: We're also going to need to keep these somehow organized for best iteration order
	components: Vec<Option<T>>,
	generations: Vec<u32>,

	available: VecDeque<usize>,
}

pub(crate) trait DynamicStorage: Send + Sync + DynamicStorageClone + 'static {
	fn insert_box(&mut self, value: &Box<dyn Any>) -> ComponentId;
	fn remove(&mut self, id: ComponentId) -> bool;

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

impl<T: Component> Storage<T> {
	fn new() -> Self {
		let capacity = 512;
		Self {
			components: Vec::with_capacity(capacity),
			generations: Vec::with_capacity(capacity),

			available: VecDeque::with_capacity(64),
		}
	}

	fn insert(&mut self, t: T) -> ComponentId {
		if self.available.is_empty() {
			self.components.push(Some(t));
			self.generations.push(0);

			ComponentId {
				variant_id: T::VARIANT_ID,

				index: (self.components.len() - 1) as u32,
				generation: 0,
			}
		} else {
			let index = self.available.pop_front().unwrap();

			self.components[index] = Some(t);
			self.generations[index] += 1;
			let generation = self.generations[index];

			ComponentId {
				variant_id: T::VARIANT_ID,

				index: index as u32,
				generation,
			}
		}
	}

	fn get(&self, id: ComponentId) -> Option<&T> {
		assert_eq!(T::VARIANT_ID, id.variant_id);

		let index = id.index as usize;

		if self.components.len() <= index {
			return None;
		}

		if self.generations[index] != id.generation {
			return None;
		}

		self.components[index].as_ref()
	}

	fn get_mut(&mut self, id: ComponentId) -> Option<&mut T> {
		assert_eq!(T::VARIANT_ID, id.variant_id);

		let index = id.index as usize;

		if self.components.len() - 1 < index {
			return None;
		}

		if self.generations[index] != id.generation {
			return None;
		}

		self.components[index].as_mut()
	}
}

impl<T: Component> DynamicStorage for Storage<T> {
	fn remove(&mut self, id: ComponentId) -> bool {
		assert_eq!(T::VARIANT_ID, id.variant_id);

		let index = id.index as usize;

		if self.components.len() - 1 < index {
			return false;
		}

		if self.generations[index] != id.generation {
			return false;
		}

		self.available.push_back(index);
		self.components[index].take().is_some()
	}

	fn insert_box(&mut self, value: &Box<dyn Any>) -> ComponentId {
		let t: &T = value.downcast_ref().unwrap();
		self.insert(t.clone())
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
	map: HashMap<ComponentVariantId, RwLock<Box<dyn DynamicStorage>>>,
	pub variants: Vec<ComponentVariant>,
}

impl ComponentsContainer {
	pub fn new(variants: Vec<ComponentVariant>) -> Self {
		let mut map = HashMap::with_capacity(variants.len());
		for v in variants.iter() {
			map.insert(v.id, RwLock::new((v.create_storage)()));
		}
		Self { map, variants }
	}

	pub fn read_id(&self, variant_id: ComponentVariantId) -> Option<ReadStorage> {
		Some(ReadStorage {
			read: self.map.get(&variant_id)?.read().unwrap(),
		})
	}

	pub fn read<T: Component>(&self) -> Option<ReadStorage> {
		self.read_id(T::VARIANT_ID)
	}

	pub fn write_id(&self, variant_id: ComponentVariantId) -> Option<WriteStorage> {
		Some(WriteStorage {
			write: self.map.get(&variant_id)?.write().unwrap(),
		})
	}

	pub fn write<T: Component>(&self) -> Option<WriteStorage> {
		self.write_id(T::VARIANT_ID)
	}
}

impl Clone for ComponentsContainer {
	fn clone(&self) -> Self {
		let mut map = HashMap::with_capacity(self.map.len());
		self.map.iter().for_each(|(key, value)| {
			let read = value.read().unwrap();
			map.insert(*key, RwLock::new(read.clone_to_box()));
		});
		Self {
			map,
			variants: self.variants.clone(),
		}
	}
}

pub struct ReadStorage<'a> {
	read: RwLockReadGuard<'a, Box<dyn DynamicStorage>>,
}

impl<'a> ReadStorage<'a> {
	pub fn get<T: Component>(&self, id: ComponentId) -> Option<&T> {
		self.read
			.as_any()
			.downcast_ref::<Storage<T>>()
			.expect("Incorrect usage of ReadStorage")
			.get(id)
	}
}

pub struct WriteStorage<'a> {
	write: RwLockWriteGuard<'a, Box<dyn DynamicStorage>>,
}

impl<'a> WriteStorage<'a> {
	pub fn insert<T: Component>(&mut self, t: T) -> ComponentId {
		self.write
			.as_any_mut()
			.downcast_mut::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.insert(t)
	}

	pub fn insert_box(&mut self, value: &Box<dyn Any>) -> ComponentId {
		self.write.insert_box(value)
	}

	pub fn get<T: Component>(&self, id: ComponentId) -> Option<&T> {
		self.write
			.as_any()
			.downcast_ref::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.get(id)
	}

	pub fn get_mut<T: Component>(&mut self, id: ComponentId) -> Option<&mut T> {
		self.write
			.as_any_mut()
			.downcast_mut::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.get_mut(id)
	}

	pub fn remove(&mut self, id: ComponentId) -> bool {
		self.write.remove(id)
	}
}
