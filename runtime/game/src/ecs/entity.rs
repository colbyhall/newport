use engine::Uuid;

use super::{
	ComponentId,
	ComponentVariantId,
};

use std::collections::HashMap;

pub type Entity = Uuid;

#[derive(Default, Clone)]
pub struct EntityInfo {
	pub components: HashMap<ComponentVariantId, ComponentId>,
}
