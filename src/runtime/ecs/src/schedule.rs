use crate::{System, World};

enum ScheduleEntry {
	Single(Box<dyn System>),
	Multiple(Vec<Box<dyn System>>),
}

impl Clone for ScheduleEntry {
	fn clone(&self) -> Self {
		match self {
			ScheduleEntry::Single(system) => ScheduleEntry::Single(system.clone_to_box()),
			ScheduleEntry::Multiple(systems) => {
				let mut result = Vec::with_capacity(systems.len());
				systems.iter().for_each(|it| result.push(it.clone_to_box()));
				ScheduleEntry::Multiple(result)
			}
		}
	}
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

#[derive(Default, Clone)]
pub struct Schedule {
	entries: Vec<ScheduleEntry>,
}

impl Schedule {
	pub fn builder() -> ScheduleBuilder {
		ScheduleBuilder {
			entries: Vec::new(),
		}
	}

	pub async fn execute(&self, world: &World, dt: f32) {
		for entry in self.entries.iter() {
			match entry {
				ScheduleEntry::Single(single) => single.run(world, dt).await,
				ScheduleEntry::Multiple(entries) => {
					let future = sync::future::join_all(entries.iter().map(|it| it.run(world, dt)));
					future.await;
				}
			}
		}
	}
}
