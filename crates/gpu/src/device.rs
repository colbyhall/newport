use crate::{
	api,

	GraphicsCommandBuffer,
	Receipt,
	Texture,
};

use std::sync::Arc;

#[derive(Clone)]
pub struct Device(pub(crate) Arc<api::Device>);

impl Device {
	pub fn acquire_backbuffer(&self) -> Texture {
		Texture(self.0.acquire_backbuffer())
	}

	pub fn submit_graphics(
		&self,
		mut command_buffers: Vec<GraphicsCommandBuffer>,
		wait_on: &[Receipt],
	) -> Receipt {
		let mut api_buffers = Vec::with_capacity(command_buffers.len());
		command_buffers
			.drain(..)
			.for_each(|x| api_buffers.push(x.0));

		self.0.submit_graphics(api_buffers, wait_on)
	}

	pub fn display(&self, wait_on: &[Receipt]) {
		self.0.display(wait_on)
	}

	pub fn wait_for_idle(&self) {
		self.0.wait_for_idle()
	}
}
