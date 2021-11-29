use crate::{api, Device};

use platform::winit::window::Window;

use std::sync::Arc;

#[derive(Debug)]
pub enum InstanceCreateError {
	FailedToLoadLibrary,
	IncompatibleDriver,
	Unknown,
}

#[derive(Clone)]
pub struct Instance(Arc<api::Instance>);

impl Instance {
	pub fn new() -> Result<Self, InstanceCreateError> {
		let inner = api::Instance::new()?;
		Ok(Self(inner))
	}

	pub fn create_device(&self, window: Option<&Window>) -> crate::Result<Device> {
		let inner = api::Device::new(self.0.clone(), window)?;
		Ok(Device(inner))
	}
}
