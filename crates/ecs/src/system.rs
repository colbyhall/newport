use crate::World;
use std::any::type_name;
use sync::async_trait;

#[async_trait]
pub trait System: BoxSystemClone + 'static + Send + Sync{
	fn name(&self) -> &'static str {
		type_name::<Self>()
	}

	async fn run(&self, world: &World, dt: f32);
}

pub trait BoxSystemClone {
	fn clone_to_box(&self) -> Box<dyn System>;
}

impl<T: System + Clone> BoxSystemClone for T {
	fn clone_to_box(&self) -> Box<dyn System> {
		Box::new(self.clone())
	}
}