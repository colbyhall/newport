use crate::{
    entity::{ Entity, EntityData },
    component::{ ComponentMap, ComponentId, Component },
    query::{ QueryFromComponents, Query },

    system::SystemRegister,
};

#[cfg(feature = "editable")]
use newport_editor::Builder;

use std::{
    any::TypeId,
    cmp::Ordering,
};

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
    pub(crate) entities: SlotMap<Entity, EntityData>,
    components: ComponentMap,
    systems:    Vec<SystemRegister>
}

impl World {
    pub fn new(mut systems: Vec<SystemRegister>) -> Self {
        systems.sort_by(|a, b|{
            let a_depends_on_b = a.depends_on.iter().find(|name| **name == b.name).is_some();
            let b_depends_on_a = b.depends_on.iter().find(|name| **name == a.name).is_some();

            assert_ne!(a_depends_on_b, b_depends_on_a, "Circular dependency");
            if a_depends_on_b {
                Ordering::Less
            } else if b_depends_on_a {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        Self {
            entities:   SlotMap::with_key(),
            components: ComponentMap::new(),
            systems:    systems,
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

    #[cfg(feature = "editable")]
    pub fn edit(&mut self, entity: Entity, builder: &mut Builder) {
        let entity_data = self.entities.get_mut(entity);
        if entity_data.is_none() {
            return;
        }

        let entity_data = entity_data.unwrap();
        for comp in entity_data.components.iter() {
            self.components.edit(comp, builder);
        }
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.entities.keys().collect()
    }

    pub fn simulate(&mut self, dt: f32) {
        let systems = self.systems.clone();
        for it in systems.iter() {
            if it.active {
                (it.func)(self, dt);
            }
        }
    }
}