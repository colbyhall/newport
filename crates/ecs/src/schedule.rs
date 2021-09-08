use crate::{
	System,
	World,
};

enum ScheduleEntry {
	Single(Box<dyn System>),
	Multiple(Vec<Box<dyn System>>),
}

pub struct ScheduleBuilder {
	entries: Vec<ScheduleEntry>,
}

impl ScheduleBuilder {
	pub fn single(mut self, system: Box<dyn System>) -> Self {
		self.entries.push(ScheduleEntry::Single(system));
		self
	}

	pub fn multiple(mut self, systems: Vec<Box<dyn System>>) -> Self {
		self.entries.push(ScheduleEntry::Multiple(systems));
		self
	}

	pub fn spawn(self) -> Schedule {
		Schedule {
			entries: self.entries,
		}
	}
}

pub struct Schedule {
	entries: Vec<ScheduleEntry>,
}

impl Schedule {
	pub fn builder() -> ScheduleBuilder {
		ScheduleBuilder {
			entries: Vec::new(),
		}
	}

	pub fn execute(&self, world: &World) {
		for entry in self.entries.iter() {
			match entry {
				ScheduleEntry::Single(single) => single.run(world),
				ScheduleEntry::Multiple(_) => todo!(),
			}
		}
	}
}
