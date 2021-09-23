use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use serde::{
	self,
	bincode,
	Deserialize,
	Serialize,
};

use engine::warn;

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

use engine::Engine;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct VariantId(u32);

impl VariantId {
	pub const fn new(name: &'static str) -> Self {
		const FNV_OFFSET_BASIC: u32 = 2166136261;
		// const FNV_PRIME: u32 = 16777619;

		const fn hash_rec(name: &'static str, index: usize, hash: u32) -> u32 {
			// let hash = hash * FNV_PRIME;
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

pub trait Component: 'static + Sized + Sync + Send + Clone {
	const VARIANT_ID: VariantId = VariantId::new(type_name::<Self>());

	const CAN_SAVE: bool;
	const SINGLETON: bool;
}

pub trait Singleton {}

impl<T> Component for T
where
	T: Sync + Send + Sized + Clone + 'static,
{
	default const VARIANT_ID: VariantId = VariantId::new(type_name::<Self>());

	default const CAN_SAVE: bool = true;

	default const SINGLETON: bool = false;
}

impl<T> Component for T
where
	T: Sync + Send + Sized + Clone + 'static + Singleton,
{
	default const VARIANT_ID: VariantId = VariantId::new(type_name::<Self>());

	default const CAN_SAVE: bool = false;

	default const SINGLETON: bool = true;
}

#[derive(Clone)]
pub struct ComponentVariant {
	pub name: &'static str,

	pub variant_id: VariantId,
	pub can_save: bool,

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
			can_save: T::CAN_SAVE,

			create_storage: create_storage::<T>,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
	variant_id: VariantId,

	index: u32,
	generation: u32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Storage<T> {
	// SPEED: We're also going to need to keep these somehow organized for best iteration order
	#[serde(bound(deserialize = "T: Deserialize<'de>"))]
	components: Vec<Option<T>>,
	generations: Vec<u32>,

	available: VecDeque<usize>,
}

pub(crate) trait DynamicStorage: Send + Sync + DynamicStorageClone + 'static {
	fn remove(&mut self, id: ComponentId) -> bool;

	fn serialize(&self) -> Vec<u8>;
	fn deserialize(&mut self, bincode: &[u8]);

	fn can_save(&self) -> bool;

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

	fn can_save(&self) -> bool {
		T::CAN_SAVE
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	default fn serialize(&self) -> Vec<u8> {
		Vec::default()
	}

	default fn deserialize(&mut self, _bincode: &[u8]) {}
}

impl<T: Component + Serialize + DeserializeOwned> DynamicStorage for Storage<T> {
	fn serialize(&self) -> Vec<u8> {
		let mut result = bincode::serialize(self).expect("Somehow failed to serialize");
		result.push(0); // EOF
		result
	}

	fn deserialize(&mut self, bincode: &[u8]) {
		*self = bincode::deserialize(bincode).expect("Somehow failed to deserialize");
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

impl Serialize for ComponentsContainer {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut seq = serializer.serialize_map(Some(self.map.len()))?;
		for (key, value) in self.map.iter() {
			let read = value.read().unwrap();

			if read.can_save() {
				seq.serialize_key(key)?;
				seq.serialize_value(&read.serialize())?;
			}
		}
		seq.end()
	}
}

impl<'de> Deserialize<'de> for ComponentsContainer {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let raw_data: HashMap<VariantId, Vec<u8>> = Deserialize::deserialize(deserializer)?;
		let variants: Vec<ComponentVariant> = Engine::register();

		let mut map = HashMap::with_capacity(raw_data.len());
		for (key, value) in raw_data.iter() {
			let variant = match variants.iter().find(|v| v.variant_id == *key) {
				Some(variant) => variant,
				None => {
					warn!("[ECS] Failed to deserialize component variant due to component variant {:?} not existing", *key);
					continue;
				}
			};

			if !variant.can_save {
				warn!("[ECS] Found component \"{}\" save data even though it is marked as can not save", variant.name);
				continue;
			}

			let mut storage = (variant.create_storage)();
			storage.deserialize(value);

			map.insert(*key, RwLock::new(storage));
		}

		Ok(Self { map, variants })
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
