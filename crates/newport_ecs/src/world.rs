use crate::entity::{ Entity, EntityData };
use crate::component::{ ComponentMap, ComponentId, Component };
use crate::query::{ QueryFromComponents, Query };

use std::any::TypeId;

use slotmap::SlotMap;

pub struct EntityBuilder<'a> {
    components: Vec<ComponentId>,
    world:      &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    fn new(world: &'a mut World) -> Self {
        Self {
            components: Vec::with_capacity(16),
            world:      world,
        }
    }

    pub fn with<T: Component>(mut self, t: T) -> Self {
        let id = self.world.components.insert(t);
        self.components.push(id);
        self
    }

    pub fn finish(self) -> Entity {
        self.world.entities.insert(EntityData{
            components: self.components,
        })
    }
}

pub struct World {
    pub(crate) entities:   SlotMap<Entity, EntityData>,
    components: ComponentMap,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities:   SlotMap::with_key(),
            components: ComponentMap::new(),
        }
    }

    pub fn create(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    pub fn destroy(&mut self, entity: Entity) -> bool {
        let data = self.entities.get(entity);
        if data.is_none() {
            return false;
        }
        let data = data.unwrap();

        for c in data.components.iter() {
            self.components.remove(c);
        }

        self.entities.remove(entity);

        true
    }

    pub fn find<T: Component>(&self, entity: Entity) -> Option<&T> {
        let entity_data = self.entities.get(entity)?;
        let variant = TypeId::of::<T>();

        for id in entity_data.components.iter() {
            if id.variant == variant {
                return self.components.find(id);
            }
        }

        None
    }

    pub fn find_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let entity_data = self.entities.get_mut(entity)?;
        let variant = TypeId::of::<T>();

        for id in entity_data.components.iter() {
            if id.variant == variant {
                return self.components.find_mut(id);
            }
        }

        None
    }

    pub fn query(&mut self) -> QueryFromComponents {
        Query::from_components(self)
    }
}