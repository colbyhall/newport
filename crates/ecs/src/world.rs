use crate::{
	ComponentVariant,
	ComponentsContainer,
	EntitiesContainer,
};

#[derive(Default)]
pub struct World {
	pub(crate) entities: EntitiesContainer,
	pub(crate) components: ComponentsContainer,
}

impl World {
	pub fn new(component_variants: Vec<ComponentVariant>) -> Self {
		Self {
			entities: EntitiesContainer::new(),
			components: ComponentsContainer::new(component_variants),
		}
	}
}
