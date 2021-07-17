use crate::component::Component;
use crate::entity::Entity;
use crate::world::World;

use std::any::TypeId;
use std::slice::{Iter, IterMut};

pub struct QueryFromComponents<'a> {
    world: &'a mut World,

    variants: Vec<TypeId>,
}

impl<'a> QueryFromComponents<'a> {
    pub fn with<T: Component>(mut self) -> Self {
        self.variants.push(TypeId::of::<T>());
        self
    }

    pub fn build(self) -> Query {
        println!("{:?}", self.variants);
        let mut found = Vec::with_capacity(128);
        'outer: for (e, data) in self.world.entities.iter() {
            for v in self.variants.iter() {
                let has = data.components.iter().find(|c| *v == c.variant).is_some();
                if !has {
                    continue 'outer;
                }
            }
            found.push(e);
        }

        Query { found: found }
    }
}

pub struct Query {
    found: Vec<Entity>,
}

impl Query {
    pub(crate) fn from_components(world: &mut World) -> QueryFromComponents {
        QueryFromComponents {
            world: world,

            variants: Vec::with_capacity(8),
        }
    }

    pub fn iter(&self) -> Iter<Entity> {
        self.found.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Entity> {
        self.found.iter_mut()
    }
}
