use crate::{api, Device, Gpu, MemoryType, Result};

use std::{marker::PhantomData, sync::Arc};

use bitflags::bitflags;

bitflags! {
	pub struct BufferUsage: u32 {
		const TRANSFER_SRC      = 0b000001;
		const TRANSFER_DST      = 0b000010;
		const VERTEX            = 0b000100;
		const INDEX             = 0b001000;
		const CONSTANTS         = 0b010000;
	}
}

#[derive(Clone)]
pub struct BufferBuilder<'a, T: Sized> {
	pub(crate) usage: BufferUsage,
	pub(crate) memory: MemoryType,
	pub(crate) len: usize,
	pub(crate) device: Option<&'a Device>,
	pub(crate) phantom: PhantomData<T>,
}

impl<'a, T: Sized> BufferBuilder<'a, T> {
	pub fn device(mut self, device: &'a Device) -> Self {
		self.device = Some(device);
		self
	}

	pub fn spawn(self) -> Result<Buffer<T>> {
		let device = match self.device {
			Some(device) => device,
			None => Gpu::device(),
		};

		Ok(Buffer {
			api: api::Buffer::new(
				device.0.clone(),
				self.usage,
				self.memory,
				std::mem::size_of::<T>() * self.len,
			)?,
			phantom: PhantomData,
			len: self.len,
		})
	}
}

#[derive(Clone)]
pub struct Buffer<T: Sized> {
	pub(crate) api: Arc<api::Buffer>,
	pub(crate) phantom: PhantomData<T>,
	pub(crate) len: usize,
}

impl<T: Sized> Buffer<T> {
	pub fn new(usage: BufferUsage, memory: MemoryType, len: usize) -> Result<Buffer<T>> {
		Buffer::builder(usage, memory, len).spawn()
	}

	pub fn new_in(
		usage: BufferUsage,
		memory: MemoryType,
		len: usize,
		device: &Device,
	) -> Result<Buffer<T>> {
		Buffer::builder(usage, memory, len).device(device).spawn()
	}

	pub fn copy_to(&self, data: &[T]) -> Result<()> {
		self.api.copy_to::<T>(data)
	}

	pub fn bindless(&self) -> Option<u32> {
		self.api.bindless()
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn builder<'a>(usage: BufferUsage, memory: MemoryType, len: usize) -> BufferBuilder<'a, T> {
		BufferBuilder {
			usage,
			memory,
			len,
			device: None,
			phantom: PhantomData,
		}
	}
}
