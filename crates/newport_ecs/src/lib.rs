#![feature(trait_alias)]
#![feature(specialization)]
#![allow(incomplete_features)]

pub mod world;
pub mod entity;
pub mod component;
pub mod query;

#[cfg(test)]
mod test;

pub use crate::{
    world::World,
    entity::Entity,
    component::Component
};