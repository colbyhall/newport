use std::any::{
	Any,
	TypeId,
};
use std::boxed::Box;
use std::collections::{
	HashMap,
	VecDeque,
};

pub trait Component: 'static + Send + Sync {
	const TRANSIENT: bool;
}

impl<T> Component for T
where
	T: Send + Sync + 'static,
{
	default const TRANSIENT: bool = true;
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct ComponentId {
	pub variant: TypeId,

	pub index: u32, // Crunch these down to u32's to save on space
	pub generation: u32,
}

// Essentially this is a SOA slotmap
struct ComponentStorage<T: Component> {
	// SPEED: This could be problematic considering how the option state is stored.
	//
	// We're also going to need to keep these somehow organized for best iteration order
	components: Vec<Option<T>>,
	generations: Vec<u32>,

	available: VecDeque<u32>, // Indices into the components vector
}

impl<T: Component> ComponentStorage<T> {
	fn new() -> Self {
		let capacity = 512;
		Self {
			components: Vec::with_capacity(capacity),
			generations: Vec::with_capacity(capacity),

			available: VecDeque::with_capacity(64),
		}
	}

	fn insert(&mut self, t: T) -> ComponentId {
		let variant = TypeId::of::<T>();

		if self.available.is_empty() {
			self.components.push(Some(t));
			self.generations.push(0);

			ComponentId {
				variant: variant,

				index: (self.components.len() - 1) as u32,
				generation: 0,
			}
		} else {
			let index = self.available.pop_front().unwrap();

			self.components[index as usize] = Some(t);
			self.generations[index as usize] += 1;
			let generation = self.generations[index as usize];

			ComponentId {
				variant,
				index,
				generation,
			}
		}
	}

	fn remove(&mut self, id: &ComponentId) -> Option<T> {
		assert_eq!(TypeId::of::<T>(), id.variant);

		let index = id.index as usize;

		if self.components.len() - 1 < index {
			return None;
		}

		if self.generations[index] != id.generation {
			return None;
		}

		self.components[index].take()
	}

	fn find(&self, id: &ComponentId) -> Option<&T> {
		assert_eq!(TypeId::of::<T>(), id.variant);

		let index = id.index as usize;

		if self.components.len() - 1 < index {
			return None;
		}

		if self.generations[index] != id.generation {
			return None;
		}

		self.components[index].as_ref()
	}

	fn find_mut(&mut self, id: &ComponentId) -> Option<&mut T> {
		assert_eq!(TypeId::of::<T>(), id.variant);

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

struct ComponentMapEntry {
	storage: Box<dyn Any>,
	remove: fn(&mut Box<dyn Any>, &ComponentId) -> bool, // The api does not require that we know type of a component at removal so we must keep a ptr to the drop method
}

impl ComponentMapEntry {
	fn new<T: Component>() -> Self {
		// We won't know the type of a component when its removed. So we cache a function that does it for us since we know the type now
		fn remove<T: Component>(boxed_storage: &mut Box<dyn Any>, id: &ComponentId) -> bool {
			let storage = boxed_storage.downcast_mut::<ComponentStorage<T>>().unwrap();
			storage.remove(id).is_some()
		}

		Self {
			storage: Box::new(ComponentStorage::<T>::new()),
			remove: remove::<T>,
		}
	}
}

pub(crate) struct ComponentMap {
	map: HashMap<TypeId, ComponentMapEntry>,
}

impl ComponentMap {
	pub fn new() -> Self {
		Self {
			map: HashMap::with_capacity(64),
		}
	}

	pub fn insert<T: Component>(&mut self, t: T) -> ComponentId {
		let variant = TypeId::of::<T>();

		// Find or create the ComponentMapEntry
		let entry = {
			let found = self.map.get_mut(&variant);
			if found.is_none() {
				self.map.insert(variant, ComponentMapEntry::new::<T>());
				self.map.get_mut(&variant).unwrap()
			} else {
				found.unwrap()
			}
		};

		let storage = entry.storage.downcast_mut::<ComponentStorage<T>>().unwrap();
		storage.insert(t)
	}

	pub fn remove(&mut self, id: &ComponentId) -> bool {
		// Find  the boxed component storage
		let entry = self.map.get_mut(&id.variant);
		if entry.is_none() {
			return false;
		}
		let entry = entry.unwrap();

		(entry.remove)(&mut entry.storage, id)
	}

	pub fn find<T: Component>(&self, id: &ComponentId) -> Option<&T> {
		let entry = self.map.get(&id.variant);
		if entry.is_none() {
			return None;
		}
		let entry = entry.unwrap();

		let storage = entry.storage.downcast_ref::<ComponentStorage<T>>().unwrap();
		storage.find(id)
	}

	pub fn find_mut<T: Component>(&mut self, id: &ComponentId) -> Option<&mut T> {
		let entry = self.map.get_mut(&id.variant);
		if entry.is_none() {
			return None;
		}
		let entry = entry.unwrap();

		let storage = entry.storage.downcast_mut::<ComponentStorage<T>>().unwrap();
		storage.find_mut(id)
	}

	#[cfg(feature = "editable")]
	pub fn edit(&mut self, id: &ComponentId, builder: &mut Builder) {
		let entry = self.map.get_mut(&id.variant);
		if entry.is_none() {
			return;
		}
		let entry = entry.unwrap();

		if entry.edit.is_none() {
			return;
		}

		let edit = entry.edit.unwrap();
		edit(&mut entry.storage, id, builder)
	}
}
