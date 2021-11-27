use engine::Uuid;

use super::{
	ComponentId,
	ComponentVariantId,
};

use std::collections::HashMap;

pub type EntityId = Uuid;

#[derive(Default, Clone)]
pub struct EntityInfo {
	pub components: HashMap<ComponentVariantId, ComponentId>,
}

#[derive(Clone)]
pub struct Entity {
	pub id: EntityId,
	pub info: EntityInfo,
}
