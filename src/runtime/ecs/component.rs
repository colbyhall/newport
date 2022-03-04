use {
	crate::{
		Entity,
		EntityInfo,
		World,
	},
	engine::Engine,
	serde::{
		self,
		de::DeserializeOwned,
		ron,
		Deserialize,
		Serialize,
	},
	std::{
		any::{
			type_name,
			Any,
		},
		cell::{
			Ref,
			RefCell,
			RefMut,
		},
		collections::{
			HashMap,
			VecDeque,
		},
		marker::PhantomData,
		sync::{
			RwLock,
			RwLockReadGuard,
			RwLockWriteGuard,
		},
	},
};

pub trait Component:
	Sync + Send + Sized + Clone + Serialize + DeserializeOwned + Default + 'static
{
	const VARIANT_ID: ComponentId = ComponentId::new(type_name::<Self>());

	fn variant() -> ComponentVariant {
		fn create_storage<T: Component>() -> Box<dyn DynamicStorage> {
			Box::new(Storage::<T>::new())
		}

		fn parse_value<T: Component>(value: ron::Value) -> ron::Result<Box<dyn Any>> {
			let t: T = value.into_rust()?;
			Ok(Box::new(t))
		}

		fn default<T: Component>() -> Box<dyn Any> {
			Box::new(T::default())
		}

		ComponentVariant {
			name: type_name::<Self>()
				.rsplit_once("::")
				.unwrap_or(("", type_name::<Self>()))
				.1,
			id: Self::VARIANT_ID,

			create_storage: create_storage::<Self>,
			parse_value: parse_value::<Self>,
			default: default::<Self>,
		}
	}

	#[allow(unused_variables)]
	fn on_added(world: &World, entity: Entity, storage: &mut WriteStorage<Self>) {}
}

#[derive(Clone)]
pub struct ComponentVariant {
	pub name: &'static str,

	pub id: ComponentId,

	create_storage: fn() -> Box<dyn DynamicStorage>,
	pub parse_value: fn(value: ron::Value) -> ron::Result<Box<dyn Any>>,
	pub default: fn() -> Box<dyn Any>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ComponentId(u32);

impl ComponentId {
	pub const fn new(name: &'static str) -> Self {
		const FNV_OFFSET_BASIC: u64 = 2166136261;
		const FNV_PRIME: u64 = 16777619;

		let mut hash = FNV_OFFSET_BASIC;
		let mut index = 0;
		while index < name.len() {
			hash = hash.wrapping_mul(FNV_PRIME);
			hash ^= name.as_bytes()[index] as u64;
			index += 1;
		}

		Self(hash as u32)
	}

	pub const fn to_mask(self) -> u128 {
		1 << (self.0 as usize & (EntityInfo::MAX_COMPONENT_TYPES - 1)) as u128
	}
}

pub(crate) trait DynamicStorage: Send + Sync + DynamicStorageClone + 'static {
	fn insert_box(&mut self, entity: Entity, value: &Box<dyn Any>);
	fn remove(&mut self, entity: Entity) -> bool;
	fn contains(&self, entity: Entity) -> bool;

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
	components: Vec<Option<RefCell<T>>>,
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
			self.components.push(Some(RefCell::new(t)));
			index
		} else {
			let index = self.available.pop_front().unwrap();
			self.components[index] = Some(RefCell::new(t));
			index
		};
		self.entity_to_index.insert(entity, index);
	}

	fn get(&self, entity: Entity) -> Option<Ref<T>> {
		let index = self
			.entity_to_index
			.get(&entity)
			.cloned()
			.unwrap_or(self.components.len());

		if self.components.len() <= index {
			return None;
		}

		let result = self.components[index].as_ref()?;
		Some(result.borrow())
	}

	fn get_mut(&self, entity: Entity) -> Option<RefMut<T>> {
		let index = self
			.entity_to_index
			.get(&entity)
			.cloned()
			.unwrap_or(self.components.len());

		if self.components.len() <= index {
			return None;
		}

		let result = self.components[index].as_ref()?;
		Some(result.borrow_mut())
	}
}

unsafe impl<T: Component> Sync for Storage<T> {}

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

	fn contains(&self, entity: Entity) -> bool {
		self.entity_to_index.contains_key(&entity)
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
	map: HashMap<ComponentId, RwLock<Box<dyn DynamicStorage>>>,
}

impl ComponentsContainer {
	pub fn new() -> Self {
		let variants: &[ComponentVariant] = Engine::register();
		let map = variants
			.iter()
			.map(|v| (v.id, RwLock::new((v.create_storage)())))
			.collect();
		Self { map }
	}

	pub fn read_id(&self, variant_id: ComponentId) -> AnyReadStorage<'_> {
		AnyReadStorage {
			read: self
				.map
				.get(&variant_id)
				.expect("Unregistered component type")
				.read()
				.unwrap(),
		}
	}

	pub fn read<T: Component>(&self) -> ReadStorage<'_, T> {
		ReadStorage {
			storage: self.read_id(T::VARIANT_ID),
			phantom: PhantomData,
		}
	}

	pub fn write_id(&self, variant_id: ComponentId) -> AnyWriteStorage<'_> {
		AnyWriteStorage {
			write: self
				.map
				.get(&variant_id)
				.expect("Unregistered component type")
				.write()
				.unwrap(),
		}
	}

	pub fn write<'a, T: Component>(&'a self, world: &'a World) -> WriteStorage<'a, T> {
		WriteStorage {
			storage: self.write_id(T::VARIANT_ID),
			phantom: PhantomData,
			world,
		}
	}
}

pub struct AnyReadStorage<'a> {
	read: RwLockReadGuard<'a, Box<dyn DynamicStorage>>,
}

impl<'a> AnyReadStorage<'a> {
	pub fn get<T: Component>(&self, entity: Entity) -> Option<Ref<T>> {
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
	pub fn get(&self, entity: Entity) -> Option<Ref<T>> {
		self.storage.get(entity)
	}
}

pub struct AnyWriteStorage<'a> {
	write: RwLockWriteGuard<'a, Box<dyn DynamicStorage>>,
}

impl<'a> AnyWriteStorage<'a> {
	pub(crate) fn insert<T: Component>(&mut self, entity: Entity, t: T) {
		self.write
			.as_any_mut()
			.downcast_mut::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.insert(entity, t)
	}

	// FIXME: Use this for deserialization
	#[allow(dead_code)]
	pub(crate) fn insert_box(&mut self, entity: Entity, value: &Box<dyn Any>) {
		self.write.insert_box(entity, value)
	}

	pub fn contains(&self, entity: Entity) -> bool {
		self.write.contains(entity)
	}

	pub fn get<T: Component>(&self, entity: Entity) -> Option<Ref<T>> {
		self.write
			.as_any()
			.downcast_ref::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.get(entity)
	}

	pub fn get_mut<T: Component>(&self, entity: Entity) -> Option<RefMut<T>> {
		self.write
			.as_any()
			.downcast_ref::<Storage<T>>()
			.expect("Incorrect usage of WriteStorage")
			.get_mut(entity)
	}

	pub(crate) fn remove(&mut self, entity: Entity) -> bool {
		self.write.remove(entity)
	}
}

pub struct WriteStorage<'a, T: Component> {
	pub(crate) storage: AnyWriteStorage<'a>,
	world: &'a World,
	phantom: PhantomData<T>,
}

impl<'a, T: Component> WriteStorage<'a, T> {
	pub fn get(&self, entity: Entity) -> Option<Ref<T>> {
		self.storage.get(entity)
	}

	pub fn get_mut(&self, entity: Entity) -> Option<RefMut<T>> {
		self.storage.get_mut(entity)
	}

	pub fn contains(&self, entity: Entity) -> bool {
		self.storage.contains(entity)
	}

	pub fn get_mut_or_default(&mut self, entity: Entity) -> RefMut<T> {
		if self.contains(entity) {
			self.get_mut(entity).unwrap()
		} else {
			self.world.insert(self, entity, T::default());
			self.get_mut(entity).unwrap()
		}
	}
}
