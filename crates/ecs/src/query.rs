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

pub struct Query<'a> {
	reads: HashMap<u32, ReadStorage<'a>>,
	writes: HashMap<u32, WriteStorage<'a>>,

	entities: &'a EntitiesContainer,

	found: Vec<Entity>,
}

impl<'a> Query<'a> {
	pub fn builder() -> QueryBuilder {
		QueryBuilder {
			reads: HashSet::new(),
			writes: HashSet::new(),
		}
	}

	pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
		let info = self.entities.get_info(entity)?;

		let id = info.components.get(&T::VARIANT_ID)?;

		match self.reads.get(&T::VARIANT_ID) {
			Some(read) => read.get(*id),
			None => {
				let write = self.writes.get(&T::VARIANT_ID)?;
				write.get(*id)
			}
		}
	}

	pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
		let info = self.entities.get_info(entity)?;

		let id = info.components.get(&T::VARIANT_ID)?;

		let write = self.writes.get_mut(&T::VARIANT_ID)?;
		write.get_mut(*id)
	}

	// pub fn iter_mut(&mut self) -> impl Iterator<Item = EntityAndComponents<'a, '_>> {
	// 	self.found.iter().map(|e| EntityAndComponents {
	// 		reads: &self.reads,
	// 		writes: &mut self.writes,

	// 		entities_container: &mut self.entities,

	// 		entity: *e,
	// 	})
	// }
}

pub struct EntityAndComponents<'a: 'b, 'b> {
	reads: &'b HashMap<u32, ReadStorage<'a>>,
	writes: &'b mut HashMap<u32, WriteStorage<'a>>,

	entities_container: &'a EntitiesContainer,

	entity: Entity,
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

			entities: &world.entities,

			found,
		}
	}
}
