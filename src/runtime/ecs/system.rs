use crate::World;
use std::any::type_name;

pub trait System: BoxSystemClone + 'static + Send + Sync {
	fn name(&self) -> &'static str {
		type_name::<Self>()
	}

	fn run(&self, world: &World, dt: f32);
}

pub trait BoxSystemClone {
	fn clone_to_box(&self) -> Box<dyn System>;
}

impl<T: System + Clone> BoxSystemClone for T {
	fn clone_to_box(&self) -> Box<dyn System> {
		Box::new(self.clone())
	}
}

enum Entry {
	System(Box<dyn System>),
	Block(ScheduleBlock),
}

#[derive(Default)]
pub struct ScheduleBlock {
	entries: Vec<Entry>,
}

impl ScheduleBlock {
	pub fn new() -> Self {
		Self {
			entries: Vec::new(),
		}
	}

	pub fn system(mut self, system: impl System) -> Self {
		self.entries.push(Entry::System(Box::new(system)));
		self
	}

	pub fn block(mut self, block: impl FnOnce(ScheduleBlock) -> ScheduleBlock) -> Self {
		self.entries.push(Entry::Block(block(ScheduleBlock::new())));
		self
	}

	// TODO: Do a fiber job system
	pub(crate) fn execute(&'static self, world: &'static World, dt: f32) {
		for entry in self.entries.iter() {
			match entry {
				Entry::System(single) => single.run(world, dt),
				Entry::Block(block) => {
					block.execute(world, dt);
				}
			}
		}
	}
}
