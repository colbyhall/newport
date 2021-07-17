use crate::component::ComponentId;

use slotmap::new_key_type;

new_key_type! { pub struct Entity; }

#[derive(Debug)]
pub(crate) struct EntityData {
    pub components: Vec<ComponentId>,
}
