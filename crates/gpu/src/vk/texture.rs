use super::{
	vk_format,
	Device,
	DeviceAllocation,
};
use crate::{
	Format,
	MemoryType,
	ResourceCreateError,
	TextureUsage,
};

use ash::version::DeviceV1_0;
use ash::vk;

use std::sync::Arc;

pub struct Texture {
	pub owner: Arc<Device>,

	pub image: vk::Image,
	pub view: vk::ImageView,
	pub memory: DeviceAllocation,

	pub memory_type: MemoryType,

	pub usage: TextureUsage,
	pub format: Format,

	pub width: u32,
	pub height: u32,
	pub depth: u32,

	// Index into the devices bindless array
	pub bindless: Option<u32>,
}

impl Texture {
	pub fn new(
		owner: Arc<Device>,
		memory_type: MemoryType,
		usage: TextureUsage,
		format: Format,
		width: u32,
		height: u32,
		depth: u32,
	) -> Result<Arc<Texture>, ResourceCreateError> {
		let mut image_type = vk::ImageType::TYPE_3D;
		if depth == 1 {
			image_type = vk::ImageType::TYPE_2D;
			if height == 1 {
				image_type = vk::ImageType::TYPE_1D;
			}
		}

		let mut image_usage = vk::ImageUsageFlags::default();
		if usage.contains(TextureUsage::TRANSFER_SRC) {
			image_usage |= vk::ImageUsageFlags::TRANSFER_SRC;
		}
		if usage.contains(TextureUsage::TRANSFER_DST) {
			image_usage |= vk::ImageUsageFlags::TRANSFER_DST;
		}
		if usage.contains(TextureUsage::SAMPLED) {
			image_usage |= vk::ImageUsageFlags::SAMPLED;
		}
		if usage.contains(TextureUsage::COLOR_ATTACHMENT) {
			image_usage |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
		}
		if usage.contains(TextureUsage::DEPTH_ATTACHMENT) {
			image_usage |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
		}

		let extent = vk::Extent3D::builder()
			.width(width)
			.height(height)
			.depth(depth)
			.build();

		let create_info = vk::ImageCreateInfo::builder()
			.image_type(image_type)
			.format(vk_format(format))
			.mip_levels(1)
			.array_layers(1)
			.samples(vk::SampleCountFlags::TYPE_1)
			.tiling(vk::ImageTiling::OPTIMAL)
			.usage(image_usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.extent(extent);

		let image = unsafe { owner.logical.create_image(&create_info, None) };
		if image.is_err() {
			return Err(ResourceCreateError::Unknown);
		}
		let image = image.unwrap();

		let requirements = unsafe { owner.logical.get_image_memory_requirements(image) };
		let memory = owner.allocate_memory(requirements, memory_type);
		if memory.is_err() {
			return Err(ResourceCreateError::OutOfMemory);
		}
		let memory = memory.unwrap();

		unsafe {
			owner
				.logical
				.bind_image_memory(image, memory.memory, memory.offset)
				.unwrap()
		};

		let mut image_view_type = vk::ImageViewType::TYPE_3D;
		if depth == 1 {
			image_view_type = vk::ImageViewType::TYPE_2D;
			if height == 1 {
				image_view_type = vk::ImageViewType::TYPE_1D;
			}
		}

		let create_info = vk::ImageViewCreateInfo::builder()
			.view_type(image_view_type)
			.image(image)
			.format(vk_format(format))
			.subresource_range(
				vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.level_count(1)
					.layer_count(1)
					.build(),
			);

		let view = unsafe { owner.logical.create_image_view(&create_info, None) };
		if view.is_err() {
			return Err(ResourceCreateError::Unknown);
		}
		let view = view.unwrap();

		// Add a weak reference to the device for bindless
		if usage.contains(TextureUsage::SAMPLED) {
			let mut bindless = owner.bindless_info.lock().unwrap();

			let found = bindless
				.textures
				.iter_mut()
				.enumerate()
				.find(|(_, x)| x.strong_count() == 0)
				.map(|(index, _)| index);

			let index = found.unwrap_or(bindless.textures.len());

			let result = Arc::new(Texture {
				owner: owner.clone(), // SPEED: Exra ref count due to mutex lock.

				image: image,
				view: view,
				memory: memory,

				memory_type: memory_type,

				usage: usage,
				format: format,

				width: width,
				height: height,
				depth: depth,

				bindless: Some(index as u32),
			});

			let weak = Arc::downgrade(&result);
			if found.is_some() {
				bindless.textures[index] = weak;
			} else {
				bindless.textures.push(weak);
			}

			return Ok(result);
		}

		Ok(Arc::new(Texture {
			owner: owner,

			image: image,
			view: view,
			memory: memory,

			memory_type: memory_type,

			usage: usage,
			format: format,

			width: width,
			height: height,
			depth: depth,

			bindless: None,
		}))
	}

	pub fn format(&self) -> Format {
		self.format
	}
	pub fn width(&self) -> u32 {
		self.width
	}
	pub fn height(&self) -> u32 {
		self.height
	}
	pub fn depth(&self) -> u32 {
		self.depth
	}

	pub fn bindless(&self) -> Option<u32> {
		self.bindless
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		if self.usage.contains(TextureUsage::SWAPCHAIN) {
			unsafe {
				self.owner.logical.destroy_image_view(self.view, None);
			}
		} else {
			unsafe {
				self.owner.logical.destroy_image(self.image, None);
				self.owner.logical.destroy_image_view(self.view, None);
				self.owner.free_memory(self.memory);
			}
		}
	}
}
