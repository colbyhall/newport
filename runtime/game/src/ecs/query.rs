use super::ComponentVariantId;
use super::Entity;
use super::{
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
	reads: HashMap<ComponentVariantId, ReadStorage<'a>>,
	writes: HashMap<ComponentVariantId, WriteStorage<'a>>,

	world: &'a World,

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

			world: self.world,

			iter: self.found.iter(),
		}
	}

	pub fn len(&self) -> usize {
		self.found.len()
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

pub struct EntityComponentIterator<'a: 'b, 'b> {
	reads: &'b mut HashMap<ComponentVariantId, ReadStorage<'a>>,
	writes: &'b mut HashMap<ComponentVariantId, WriteStorage<'a>>,

	world: &'a World,

	iter: Iter<'b, Entity>,
}

impl<'a: 'b, 'b> Iterator for EntityComponentIterator<'a, 'b> {
	type Item = EntityComponents<'a, 'b>;

	fn next(&mut self) -> Option<Self::Item> {
		let e = self.iter.next()?;
		Some(EntityComponents {
			reads: NonNull::new(self.reads).unwrap(),
			writes: NonNull::new(self.writes).unwrap(),

			world: self.world,

			phantom: PhantomData,

			entity: *e,
		})
	}
}

pub struct EntityComponents<'a: 'b, 'b> {
	reads: NonNull<HashMap<ComponentVariantId, ReadStorage<'a>>>,
	writes: NonNull<HashMap<ComponentVariantId, WriteStorage<'a>>>,

	world: &'a World,

	phantom: PhantomData<&'b i32>,

	entity: Entity,
}

impl<'a: 'b, 'b> EntityComponents<'a, 'b> {
	pub fn get<T: Component>(&self) -> Option<&T> {
		let scene = self.world.persistent_scene.read().unwrap();
		let info = scene.entities.get(&self.entity)?;

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

	pub fn get_mut<T: Component>(&self) -> Option<&mut T> {
		let scene = self.world.persistent_scene.read().unwrap();
		let info = scene.entities.get(&self.entity)?;

		let id = info.components.get(&T::VARIANT_ID)?;

		let write = unsafe { (&mut *self.writes.as_ptr()).get_mut(&T::VARIANT_ID)? };
		write.get_mut(*id)
	}
}

pub struct QueryBuilder {
	reads: HashSet<ComponentVariantId>,
	writes: HashSet<ComponentVariantId>,
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

		let found = {
			let scene = world.persistent_scene.read().unwrap();
			scene
				.entities
				.iter()
				.filter(|(_, info)| !components.iter().any(|c| info.components.get(c).is_none()))
				.map(|(id, _)| *id)
				.collect()
		};

		Query {
			reads,
			writes,

			world,

			found,
		}
	}
}
