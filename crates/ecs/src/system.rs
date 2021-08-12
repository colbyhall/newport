use crate::World;

use std::any::Any;

pub trait System: SystemAsAny + 'static {
	fn name(&self) -> &'static str;
	fn run(&mut self, world: &World);
}

pub trait SystemAsAny {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: System + 'static> SystemAsAny for T {
	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}
}
