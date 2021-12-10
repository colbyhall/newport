use engine::Uuid;

use std::collections::HashMap;

pub type Entity = Uuid;

#[derive(Default, Clone)]
pub struct EntityInfo {
	pub components: u128,
}

impl EntityInfo {
	pub const MAX_COMPONENT_TYPES: usize = 128;
}

pub type EntityContainer = HashMap<Entity, EntityInfo>;
