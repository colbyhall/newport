use crate::EntitiesContainer;
use crate::Entity;
use crate::{
	Component,
	ReadStorage,
	World,
	WriteStorage,
};

use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::slice::Iter;

pub struct Query<'a> {
	reads: HashMap<u32, ReadStorage<'a>>,
	writes: HashMap<u32, WriteStorage<'a>>,

	entities_collection: &'a EntitiesContainer,

	found: Vec<Entity>,
}

impl<'a> Query<'a> {
	pub fn builder() -> QueryBuilder {
		QueryBuilder {
			reads: HashSet::new(),
			writes: HashSet::new(),
		}
	}

	pub fn iter(&mut self) -> EntityComponentIterator<'a, '_> {
		EntityComponentIterator {
			reads: &mut self.reads,
			writes: &mut self.writes,

			entities_collection: self.entities_collection,

			iter: self.found.iter(),
		}
	}
}

pub struct EntityComponentIterator<'a: 'b, 'b> {
	reads: &'b mut HashMap<u32, ReadStorage<'a>>,
	writes: &'b mut HashMap<u32, WriteStorage<'a>>,

	entities_collection: &'a EntitiesContainer,

	iter: Iter<'b, Entity>,
}

impl<'a: 'b, 'b> Iterator for EntityComponentIterator<'a, 'b> {
	type Item = EntityComponents<'a, 'b>;

	fn next(&mut self) -> Option<Self::Item> {
		let e = self.iter.next()?;
		Some(EntityComponents {
			reads: NonNull::new(self.reads).unwrap(),
			writes: NonNull::new(self.writes).unwrap(),

			entities_collection: self.entities_collection,

			phantom: PhantomData,

			entity: *e,
		})
	}
}

pub struct EntityComponents<'a: 'b, 'b> {
	reads: NonNull<HashMap<u32, ReadStorage<'a>>>,
	writes: NonNull<HashMap<u32, WriteStorage<'a>>>,

	entities_collection: &'a EntitiesContainer,

	phantom: PhantomData<&'b i32>,

	entity: Entity,
}

impl<'a: 'b, 'b> EntityComponents<'a, 'b> {
	pub fn get<T: Component>(&self) -> Option<&T> {
		let info = self.entities_collection.get_info(self.entity)?;

		let id = info.components.get(&T::VARIANT_ID)?;

		unsafe {
			match self.reads.as_ref().get(&T::VARIANT_ID) {
				Some(read) => read.get(*id),
				None => {
					let write = self.writes.as_ref().get(&T::VARIANT_ID)?;
					write.get(*id)
				}
			}
		}
	}

	pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
		let info = self.entities_collection.get_info(self.entity)?;

		let id = info.components.get(&T::VARIANT_ID)?;

		let write = unsafe { self.writes.as_mut().get_mut(&T::VARIANT_ID)? };
		write.get_mut(*id)
	}
}

pub struct QueryBuilder {
	reads: HashSet<u32>,
	writes: HashSet<u32>,
}

impl QueryBuilder {
	pub fn read<T: Component>(mut self) -> Self {
		self.reads.insert(T::VARIANT_ID);
		self
	}

	pub fn write<T: Component>(mut self) -> Self {
		self.writes.insert(T::VARIANT_ID);
		self
	}

	pub fn execute(self, world: &World) -> Query {
		let mut components = HashSet::with_capacity(self.reads.len() + self.writes.len());
		let mut reads = HashMap::with_capacity(self.reads.len());
		let mut writes = HashMap::with_capacity(self.writes.len());

		for r in self.reads {
			components.insert(r);
			reads.insert(r, world.components.read_id(r).unwrap());
		}

		for w in self.writes {
			components.insert(w);
			writes.insert(w, world.components.write_id(w).unwrap());
		}

		let found = world.entities.gather_with_active(components);

		Query {
			reads,
			writes,

			entities_collection: &world.entities,

			found,
		}
	}
}
