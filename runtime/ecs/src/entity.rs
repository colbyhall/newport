use crate::{
	ComponentId,
	VariantId,
};

use std::collections::{
	HashMap,
	HashSet,
	VecDeque,
};
use std::ops::Index;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entity {
	index: u32,
	generation: u32,
}

#[derive(Default, Clone)]
pub struct EntityInfo {
	pub components: HashMap<VariantId, ComponentId>,
}

#[derive(Default, Clone)]
pub struct EntitiesContainer {
	generations: Vec<u32>,
	available: VecDeque<u32>,

	entity_info: Vec<Option<EntityInfo>>,
	active_components: Vec<HashSet<VariantId>>,
}

impl EntitiesContainer {
	pub fn new() -> Self {
		const RESERVE: usize = 1024;

		Self {
			generations: Vec::with_capacity(RESERVE),
			available: VecDeque::new(),

			entity_info: Vec::with_capacity(RESERVE),
			active_components: Vec::with_capacity(RESERVE),
		}
	}

	pub fn insert(&mut self, entity_info: EntityInfo) -> Entity {
		if let Some(index) = self.available.pop_front() {
			let mut active_components = HashSet::with_capacity(entity_info.components.len());
			for variant in entity_info.components.keys() {
				active_components.insert(*variant);
			}
			self.active_components[index as usize] = active_components;

			self.entity_info[index as usize] = Some(entity_info);
			self.generations[index as usize] += 1;

			Entity {
				index,
				generation: self.generations[index as usize],
			}
		} else {
			let mut active_components = HashSet::with_capacity(entity_info.components.len());
			for variant in entity_info.components.keys() {
				active_components.insert(*variant);
			}
			self.active_components.push(active_components);

			self.entity_info.push(Some(entity_info));
			self.generations.push(0);

			Entity {
				index: (self.entity_info.len() - 1) as u32,
				generation: 0,
			}
		}
	}

	pub fn get_info(&self, entity: Entity) -> Option<&EntityInfo> {
		let index = entity.index as usize;

		if self.entity_info.index(index).is_none() {
			return None;
		}

		if self.generations[index] != entity.generation {
			return None;
		}

		self.entity_info[index].as_ref()
	}

	pub fn gather_with_active(&self, components: HashSet<VariantId>) -> Vec<Entity> {
		let mut result = Vec::new();
		'outer: for (index, _) in self
			.entity_info
			.iter()
			.enumerate()
			.filter(|(_, info)| info.is_some())
		{
			let active = &self.active_components[index];

			for c in components.iter() {
				if !active.contains(c) {
					continue 'outer;
				}
			}

			result.push(Entity {
				index: index as u32,
				generation: self.generations[index],
			})
		}
		result
	}
}
