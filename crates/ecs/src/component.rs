use serde::{
	self,
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
#[serde(crate = "self::serde")]
pub struct VariantId(u32);

impl VariantId {
	pub const fn new(name: &'static str) -> Self {
		const FNV_OFFSET_BASIC: u32 = 2166136261;
		const FNV_PRIME: u32 = 16777619;

		const fn hash_rec(name: &'static str, index: usize, hash: u32) -> u32 {
			let hash = hash * FNV_PRIME;
			let hash = hash ^ name.as_bytes()[index] as u32;
			if index != name.len() - 1 {
				hash_rec(name, index + 1, hash)
			} else {
				hash
			}
		}

		Self(hash_rec(name, 0, FNV_OFFSET_BASIC))
	}
}

pub trait Component: 'static + Sized + Sync {
	const VARIANT_ID: VariantId = VariantId::new(type_name::<Self>());

	const CAN_SAVE: bool;
}

impl<T> Component for T
where
	T: Sync + Sized + 'static,
{
	default const VARIANT_ID: VariantId = VariantId::new(type_name::<Self>());

	default const CAN_SAVE: bool = true;
}

#[derive(Clone)]
pub struct ComponentVariant {
	pub name: &'static str,
	pub variant_id: VariantId,

	create_storage: fn() -> Box<dyn DynamicStorage>,
}

impl ComponentVariant {
	pub fn new<T: Component>() -> Self {
		fn create_storage<T: Component>() -> Box<dyn DynamicStorage> {
			Box::new(Storage::<T>::new())
		}

		Self {
			name: type_name::<T>(),
			variant_id: T::VARIANT_ID,

			create_storage: create_storage::<T>,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct ComponentId {
	variant_id: VariantId,

	index: u32,
	generation: u32,
}

struct Storage<T: Component> {
	// SPEED: We're also going to need to keep these somehow organized for best iteration order
	components: Vec<Option<T>>,
	generations: Vec<u32>,

	available: VecDeque<usize>,
}

trait DynamicStorage {
	fn remove(&mut self, id: ComponentId) -> bool;

	fn as_any_mut(&mut self) -> &mut dyn Any;
	fn as_any(&self) -> &dyn Any;
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

		if self.components.len() - 1 < index {
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

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

#[derive(Default)]
pub struct ComponentsContainer {
	map: HashMap<VariantId, RwLock<Box<dyn DynamicStorage>>>,
	pub variants: Vec<ComponentVariant>,
}

impl ComponentsContainer {
	pub fn new(variants: Vec<ComponentVariant>) -> Self {
		let mut map = HashMap::with_capacity(variants.len());
		for v in variants.iter() {
			map.insert(v.variant_id, RwLock::new((v.create_storage)()));
		}
		Self { map, variants }
	}

	pub fn read_id(&self, variant_id: VariantId) -> Option<ReadStorage> {
		Some(ReadStorage {
			read: self.map.get(&variant_id)?.read().unwrap(),
		})
	}

	pub fn read<T: Component>(&self) -> Option<ReadStorage> {
		self.read_id(T::VARIANT_ID)
	}

	pub fn write_id(&self, variant_id: VariantId) -> Option<WriteStorage> {
		Some(WriteStorage {
			write: self.map.get(&variant_id)?.write().unwrap(),
		})
	}

	pub fn write<T: Component>(&self) -> Option<WriteStorage> {
		self.write_id(T::VARIANT_ID)
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
