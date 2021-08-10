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

struct TestComponent {
	x: f32,
	y: f32,
	z: f32,
}

// fn test() {
// 	let world = World::new(Default::default());

// 	let mut query = Query::builder().write::<TestComponent>().execute(&world);
// 	for e in query.into_iter() {
// 		let result = query.get_mut::<TestComponent>(e);
// 	}
// }
