use crate::entity::Entity;

use slotmap::SlotMap;

struct EntityData {
    components: 
}

struct ComponentId {
    
}

struct ComponentContainer {

}

pub struct World {
    entities: SlotMap<Entity, EntityData>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: SlotMap::with_key()
        }
    }
}