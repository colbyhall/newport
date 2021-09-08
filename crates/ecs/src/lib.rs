#![feature(trait_alias)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![feature(const_fn)]
#![feature(const_type_name)]
#![allow(arithmetic_overflow)]

mod component;
mod entity;
mod query;
mod schedule;
mod system;
mod world;

pub use {
	component::*,
	entity::*,
	query::*,
	schedule::*,
	system::*,
	world::*,
};
