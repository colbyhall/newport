#![feature(trait_alias)]
#![feature(specialization)]
#![allow(incomplete_features)]

pub mod component;
pub mod entity;
pub mod query;
pub mod system;
pub mod world;

#[cfg(test)]
mod test;

pub use crate::{component::Component, entity::Entity, system::System, world::World};
