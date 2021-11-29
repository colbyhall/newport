use super::{
	Device,
	DeviceAllocation,
};
use crate::{
	BufferUsage,
	MemoryType,
	Result,
};

use ash::version::DeviceV1_0;
use ash::vk;

use std::ptr::copy_nonoverlapping;
use std::sync::{
	Arc,
	RwLock,
};

pub struct Buffer {
	pub owner: Arc<Device>,

	pub handle: vk::Buffer,
	pub memory: RwLock<DeviceAllocation>,
	pub size: usize,

	pub usage: BufferUsage,

	pub bindless: Option<u32>, // Index into owner bindless buffer array
}

impl Buffer {
	pub fn new(
		owner: Arc<Device>,
		usage: BufferUsage,
		memory: MemoryType,
		size: usize,
	) -> Result<Arc<Buffer>> {
		let mut vk_usage = vk::BufferUsageFlags::default();
		if usage.contains(BufferUsage::TRANSFER_SRC) {
			vk_usage |= vk::BufferUsageFlags::TRANSFER_SRC;
		}
		if usage.contains(BufferUsage::TRANSFER_DST) {
			vk_usage |= vk::BufferUsageFlags::TRANSFER_DST;
		}
		if usage.contains(BufferUsage::VERTEX) {
			vk_usage |= vk::BufferUsageFlags::VERTEX_BUFFER;
		}
		if usage.contains(BufferUsage::INDEX) {
			vk_usage |= vk::BufferUsageFlags::INDEX_BUFFER;
		}
		if usage.contains(BufferUsage::CONSTANTS) {
			vk_usage |= vk::BufferUsageFlags::STORAGE_BUFFER;
		}

		unsafe {
			let create_info = vk::BufferCreateInfo::builder()
				.size(size as vk::DeviceSize)
				.usage(vk_usage)
				.sharing_mode(vk::SharingMode::EXCLUSIVE);

			let handle = owner.logical.create_buffer(&create_info, None)?;

			// Allocate memory for buffer
			let memory = owner
				.allocate_memory(owner.logical.get_buffer_memory_requirements(handle), memory)?;
			owner.logical.bind_buffer_memory(handle, memory.memory, 0)?;

			// Add a weak reference to the device for bindless
			if usage.contains(BufferUsage::CONSTANTS) {
				let mut bindless = owner.bindless_info.lock().unwrap();

				let found = bindless
					.buffers
					.iter_mut()
					.enumerate()
					.find(|(_, x)| x.strong_count() == 0)
					.map(|(index, _)| index);

				let index = found.unwrap_or(bindless.buffers.len());

				let result = Arc::new(Buffer {
					owner: owner.clone(),

					handle,
					memory: RwLock::new(memory),
					size,

					usage,

					bindless: Some(index as u32),
				});

				let weak = Arc::downgrade(&result);
				if found.is_some() {
					bindless.buffers[index] = weak;
				} else {
					bindless.buffers.push(weak);
				}

				return Ok(result);
			}

			Ok(Arc::new(Buffer {
				owner,

				handle,
				memory: RwLock::new(memory),
				size,

				usage,

				bindless: None,
			}))
		}
	}

	pub fn copy_to<T>(&self, data: &[T]) -> Result<()> {
		let memory = self.memory.write().unwrap();

		unsafe {
			let ptr = self.owner.logical.map_memory(
				memory.memory,
				memory.offset,
				memory.size,
				vk::MemoryMapFlags::empty(),
			)?;

			copy_nonoverlapping(
				data.as_ptr() as *const u8,
				ptr as *mut u8,
				memory.size as usize,
			);

			self.owner.logical.unmap_memory(memory.memory);
		}
		Ok(())
	}

	pub fn bindless(&self) -> Option<u32> {
		self.bindless
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		unsafe {
			self.owner.logical.destroy_buffer(self.handle, None);
		}

		let memory = self.memory.write().unwrap();
		self.owner.free_memory(*memory);
	}
}
