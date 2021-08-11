#![feature(trait_alias)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![feature(const_fn)]
#![feature(const_type_name)]

mod component;
mod entity;
mod query;
mod world;

pub use {
	component::*,
	entity::*,
	query::*,
	world::*,
};
