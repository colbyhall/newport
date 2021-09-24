use engine::Uuid;

use crate::{
	ComponentId,
	ComponentVariantId,
};

use std::collections::HashMap;

pub type Entity = Uuid;

#[derive(Default, Clone)]
pub struct EntityInfo {
	pub components: HashMap<ComponentVariantId, ComponentId>,
}
