use crate::ComponentId;

use serde::{
	self,
	Deserialize,
	Serialize,
};

use std::collections::{
	HashMap,
	HashSet,
	VecDeque,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct Entity {
	index: u32,
	generation: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct EntityInfo {
	pub active: bool,
	pub name: String,
	pub components: HashMap<u32, ComponentId>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "self::serde")]
pub struct EntitiesContainer {
	in_use: Vec<bool>,
	generations: Vec<u32>,
	available: VecDeque<u32>,

	entity_info: Vec<EntityInfo>,
	active_components: Vec<HashSet<u32>>, // u32 is Variant_Id
}

impl EntitiesContainer {
	pub fn new() -> Self {
		const RESERVE: usize = 1024;

		Self {
			in_use: Vec::with_capacity(RESERVE),
			generations: Vec::with_capacity(RESERVE),
			available: VecDeque::new(),

			entity_info: Vec::with_capacity(RESERVE),
			active_components: Vec::with_capacity(RESERVE),
		}
	}

	pub fn get_info(&self, entity: Entity) -> Option<&EntityInfo> {
		let index = entity.index as usize;

		if self.in_use.len() - 1 < index {
			return None;
		}

		if self.generations[index] != entity.generation {
			return None;
		}

		if !self.in_use[index] {
			return None;
		}

		Some(&self.entity_info[index])
	}

	pub fn gather_with_active(&self, components: HashSet<u32>) -> Vec<Entity> {
		let mut result = Vec::new();
		for (index, in_use) in self.in_use.iter().enumerate() {
			if !in_use {
				continue;
			}

			let active = &self.active_components[index];

			for c in components.iter() {
				if !active.contains(&c) {
					continue;
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
