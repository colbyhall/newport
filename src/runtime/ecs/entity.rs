use engine::Uuid;

use serde::{
	Deserialize,
	Serialize,
};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Entity {
	pub(crate) id: Uuid,
	pub(crate) scene: Uuid,
}

impl Entity {
	pub fn new(scene: Uuid) -> Self {
		Self {
			id: Uuid::new(),
			scene,
		}
	}
}

#[derive(Default, Clone)]
pub struct EntityInfo {
	pub components: u128,
}

impl EntityInfo {
	pub const MAX_COMPONENT_TYPES: usize = 128;
}

pub type EntityContainer = HashMap<Entity, EntityInfo>;
