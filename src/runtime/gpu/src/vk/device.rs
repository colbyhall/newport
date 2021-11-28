use super::{
	Buffer,
	GraphicsCommandBuffer,
	Instance,
	Receipt,
	Texture,
	ENABLED_LAYER_NAMES,
};
use super::{
	RenderPass,
	Sampler,
};
use crate::{
	BufferUsage,
	Format,
	MemoryType,
	Result,
	TextureUsage,
};

use ash::extensions::khr;
use ash::version::{
	DeviceV1_0,
	InstanceV1_0,
	InstanceV1_1,
};
use ash::vk;

use std::collections::HashMap;
use std::slice::from_ref;
use std::sync::{
	Arc,
	Mutex,
	Weak,
};
use std::thread::ThreadId;

use platform::raw_window_handle::{
	HasRawWindowHandle,
	RawWindowHandle,
};
use platform::winit::window::Window;

struct Swapchain {
	// HACK: Leak the swapchain handle because it crashes when trying to free it. Probably due to it being attached to resources???
	// TODO: Maybe actually handle this?
	handle: vk::SwapchainKHR,

	backbuffers: Vec<Arc<Texture>>,
	current: Option<usize>,
}

impl Swapchain {
	fn new(device: Arc<Device>) -> Result<Self> {
		assert!(device.surface.is_some());

		let swapchain_khr = khr::Swapchain::new(&device.owner.instance, &device.logical);
		let surface_khr = khr::Surface::new(&device.owner.entry, &device.owner.instance);

		unsafe {
			let capabilities = surface_khr.get_physical_device_surface_capabilities(
				device.physical,
				device.surface.unwrap(),
			)?;
			let formats = surface_khr
				.get_physical_device_surface_formats(device.physical, device.surface.unwrap())?;

			let mut selected_format = None;
			for it in formats.iter() {
				if it.format == vk::Format::B8G8R8A8_SRGB
					&& it.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
				{
					selected_format = Some(it);
					break;
				}
			}
			let selected_format = selected_format.unwrap();

			let mut queue_family_indices = Vec::with_capacity(2);
			if device.graphics_family_index.is_some() {
				queue_family_indices.push(device.graphics_family_index.unwrap());
			}
			if device.surface_family_index.is_some() {
				queue_family_indices.push(device.surface_family_index.unwrap());
			}

			let create_info = vk::SwapchainCreateInfoKHR::builder()
				.surface(device.surface.unwrap())
				.min_image_count(capabilities.min_image_count)
				.image_format(selected_format.format)
				.image_color_space(selected_format.color_space)
				.image_extent(capabilities.current_extent)
				.image_array_layers(1)
				.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
				.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
				.queue_family_indices(&queue_family_indices[..])
				.pre_transform(capabilities.current_transform)
				.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
				.present_mode(vk::PresentModeKHR::FIFO)
				.clipped(true)
				.build();

			let handle = swapchain_khr.create_swapchain(&create_info, None)?;

			let images = swapchain_khr.get_swapchain_images(handle)?;

			let mut backbuffers = Vec::with_capacity(images.len());
			for it in images.iter() {
				let create_info = vk::ImageViewCreateInfo::builder()
					.image(*it)
					.view_type(vk::ImageViewType::TYPE_2D)
					.format(selected_format.format)
					.components(vk::ComponentMapping {
						r: vk::ComponentSwizzle::IDENTITY,
						g: vk::ComponentSwizzle::IDENTITY,
						b: vk::ComponentSwizzle::IDENTITY,
						a: vk::ComponentSwizzle::IDENTITY,
					})
					.subresource_range(vk::ImageSubresourceRange {
						aspect_mask: vk::ImageAspectFlags::COLOR,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					});

				let view = device.logical.create_image_view(&create_info, None)?;

				backbuffers.push(Arc::new(Texture {
					owner: device.clone(),

					image: *it,
					view,
					memory: DeviceAllocation::default(),

					memory_type: MemoryType::HostVisible,
					usage: TextureUsage::SWAPCHAIN,
					format: Format::BGR_U8_SRGB,

					width: capabilities.current_extent.width,
					height: capabilities.current_extent.height,
					depth: 1,

					bindless: None,
				}));
			}

			Ok(Self {
				handle,

				backbuffers,
				current: None,
			})
		}
	}
}

#[derive(Default, Copy, Clone)]
pub struct DeviceThreadInfo {
	pub graphics_pool: vk::CommandPool,
	pub compute_pool: vk::CommandPool,
	pub transfer_pool: vk::CommandPool,
}

#[derive(Default, Copy, Clone)]
pub struct DeviceAllocation {
	pub memory: vk::DeviceMemory,
	pub offset: vk::DeviceSize,
	pub size: vk::DeviceSize,
}

pub enum WorkVariant {
	Graphics(Vec<GraphicsCommandBuffer>),
}

pub struct WorkEntry {
	pub semaphore: vk::Semaphore,
	pub fence: vk::Fence,
	pub variant: WorkVariant,
	pub thread_id: ThreadId,
}

pub struct WorkContainer {
	pub last_id: usize,
	pub in_queue: HashMap<usize, WorkEntry>,
}

pub struct BindlessInfo {
	pub textures: Vec<Weak<Texture>>,
	pub null_texture: Option<Arc<Texture>>,

	pub buffers: Vec<Weak<Buffer>>,
	pub null_buffer: Option<Arc<Buffer>>,

	pub samplers: Vec<Weak<Sampler>>,
	pub null_sampler: Option<Arc<Sampler>>,
}

pub struct Device {
	pub owner: Arc<Instance>,

	pub logical: ash::Device,
	pub physical: vk::PhysicalDevice,

	pub graphics_queue: Option<Mutex<vk::Queue>>,
	pub presentation_queue: Option<Mutex<vk::Queue>>,

	pub graphics_family_index: Option<u32>,
	pub surface_family_index: Option<u32>,

	pub work: Mutex<WorkContainer>,

	#[cfg(target_os = "windows")]
	pub surface: Option<vk::SurfaceKHR>,

	swapchain: Mutex<Option<Swapchain>>,
	pub thread_info: Mutex<HashMap<ThreadId, DeviceThreadInfo>>,

	pub bindless_info: Mutex<BindlessInfo>,

	pub bindless_layout: vk::DescriptorSetLayout,
	pub bindless_pool: vk::DescriptorPool,
	pub bindless_set: vk::DescriptorSet,

	pub render_passes: Mutex<Vec<Arc<RenderPass>>>,
}

impl Device {
	pub fn allocate_memory(
		&self,
		requirements: vk::MemoryRequirements,
		memory_type: MemoryType,
	) -> Result<DeviceAllocation> {
		let property_flag = match memory_type {
			MemoryType::DeviceLocal => vk::MemoryPropertyFlags::DEVICE_LOCAL,
			MemoryType::HostVisible => {
				vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE
			}
		};
		unsafe {
			let properties = self
				.owner
				.instance
				.get_physical_device_memory_properties(self.physical);

			let mut index = None;
			for i in 0..properties.memory_type_count {
				let mut can_use = (requirements.memory_type_bits & (1 << i)) != 0;
				can_use &= properties.memory_types[i as usize].property_flags & property_flag
					!= vk::MemoryPropertyFlags::empty();

				if can_use {
					index = Some(i);
					break;
				}
			}
			let index = index.unwrap();

			let alloc_info = vk::MemoryAllocateInfo::builder()
				.allocation_size(requirements.size)
				.memory_type_index(index);

			Ok(DeviceAllocation {
				memory: self.logical.allocate_memory(&alloc_info, None)?,
				offset: 0,
				size: requirements.size,
			})
		}
	}

	pub fn free_memory(&self, allocation: DeviceAllocation) {
		unsafe {
			self.logical.free_memory(allocation.memory, None);
		}
	}

	pub fn get_or_create_render_pass(&self, attachments: &[Format]) -> Result<Arc<RenderPass>> {
		let mut render_passes = self.render_passes.lock().unwrap();
		match render_passes.iter().find(|a| a.attachments == attachments) {
			Some(render_pass) => Ok(render_pass.clone()),
			None => {
				let render_pass = RenderPass::new(self, attachments.to_vec())?;
				render_passes.push(render_pass.clone());
				Ok(render_pass)
			}
		}
	}

	fn push_work(&self, entry: WorkEntry) -> usize {
		let mut work = self.work.lock().unwrap();

		let id = work.last_id;
		work.in_queue.insert(id, entry);
		work.last_id += 1;
		id
	}

	pub fn new(instance: Arc<Instance>, window: Option<&Window>) -> Result<Arc<Self>> {
		// Find a physical device based off of some parameters
		let physical_device;
		unsafe {
			let physical_devices = instance.instance.enumerate_physical_devices()?;

			let mut selected_device = None;
			for it in physical_devices.iter() {
				let properties = instance.instance.get_physical_device_properties(*it);
				let features = instance.instance.get_physical_device_features(*it);

				// Find extensions to do bindless
				let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::default();

				let mut device_features = vk::PhysicalDeviceFeatures2 {
					p_next: &mut indexing_features
						as *mut vk::PhysicalDeviceDescriptorIndexingFeatures
						as *mut std::ffi::c_void,
					..Default::default()
				};
				instance
					.instance
					.get_physical_device_features2(*it, &mut device_features);

				// TODO: Maybe do more checking with features we actually will need like KHR Swapchain support?
				//  also maybe take something in from the builder
				let mut is_acceptable = true;
				is_acceptable &= properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
					&& features.geometry_shader == 1;
				is_acceptable &= indexing_features.descriptor_binding_partially_bound == 1
					&& indexing_features.runtime_descriptor_array == 1;
				// TODO: Update after bind for descriptor sets

				if is_acceptable {
					selected_device = Some(*it);
				}
			}

			if selected_device.is_none() {
				return Err(super::Error::ERROR_DEVICE_LOST);
			}

			physical_device = selected_device.unwrap();
		}

		// Create the surface if the builder provided one
		#[cfg(target_os = "windows")]
		let surface;
		unsafe {
			if let Some(window) = window {
				let handle = window.raw_window_handle();

				match handle {
					RawWindowHandle::Windows(handle) => {
						let surface_khr =
							khr::Win32Surface::new(&instance.entry, &instance.instance);
						let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
							.hinstance(handle.hinstance)
							.hwnd(handle.hwnd);

						surface = Some(surface_khr.create_win32_surface(&create_info, None)?);
					}
					_ => todo!(),
				}
			} else {
				surface = None;
			};
		}

		// Find the proper queue family indices
		let mut graphics_family_index = None;
		let mut surface_family_index = None;
		unsafe {
			let queue_family_properties = instance
				.instance
				.get_physical_device_queue_family_properties(physical_device);
			for (index, it) in queue_family_properties.iter().enumerate() {
				if it.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
					graphics_family_index = Some(index as u32);
				}

				if window.is_some() {
					let surface_khr = khr::Surface::new(&instance.entry, &instance.instance);
					let present_support = surface_khr
						.get_physical_device_surface_support(
							physical_device,
							index as u32,
							surface.unwrap(),
						)
						.unwrap();
					if present_support {
						surface_family_index = Some(index as u32);
					}
				}
			}
		}

		let queue_family_indices = [graphics_family_index, surface_family_index];

		// Create the logical device and the queues
		let logical_device;
		let graphics_queue;
		let presentation_queue;
		unsafe {
			// TODO: Use a custom linear or temp allocator later on when thats created
			let mut queue_create_infos = Vec::new();

			let queue_priorities = [0.0];
			for it in queue_family_indices.iter() {
				let create_info = vk::DeviceQueueCreateInfo::builder()
					.queue_family_index(it.unwrap())
					.queue_priorities(&queue_priorities);

				queue_create_infos.push(create_info.build());
			}

			let device_features = vk::PhysicalDeviceFeatures::default();
			let extensions = [b"VK_KHR_swapchain\0".as_ptr() as *const i8];

			let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures::builder()
				.descriptor_binding_partially_bound(true)
				.runtime_descriptor_array(true)
				.descriptor_binding_sampled_image_update_after_bind(true)
				.descriptor_binding_storage_buffer_update_after_bind(true);

			let create_info = vk::DeviceCreateInfo::builder()
				.push_next(&mut indexing_features)
				.queue_create_infos(&queue_create_infos[..])
				.enabled_layer_names(&ENABLED_LAYER_NAMES)
				.enabled_extension_names(&extensions)
				.enabled_features(&device_features);

			#[cfg(feature = "aftermath")]
			let mut aftermath_features = vk::DeviceDiagnosticsConfigCreateInfoNV::builder();

			#[cfg(feature = "aftermath")]
			let create_info = create_info.push_next(&mut aftermath_features);

			logical_device =
				instance
					.instance
					.create_device(physical_device, &create_info, None)?;
			graphics_queue =
				Some(logical_device.get_device_queue(graphics_family_index.unwrap(), 0));

			if let Some(surface_family_index) = surface_family_index {
				presentation_queue = Some(logical_device.get_device_queue(surface_family_index, 0));
			} else {
				presentation_queue = None;
			}
		}

		// Do the whole bindless setup thing
		let bindless_bindings = [
			vk::DescriptorSetLayoutBinding::builder()
				.binding(0)
				.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
				.descriptor_count(2048)
				.stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS)
				.build(),
			vk::DescriptorSetLayoutBinding::builder()
				.binding(1)
				.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
				.descriptor_count(2048)
				.stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS)
				.build(),
			vk::DescriptorSetLayoutBinding::builder()
				.binding(2)
				.descriptor_type(vk::DescriptorType::SAMPLER)
				.descriptor_count(2048)
				.stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS)
				.build(),
		];

		let bind_flags = [vk::DescriptorBindingFlags::PARTIALLY_BOUND_EXT
			| vk::DescriptorBindingFlags::UPDATE_AFTER_BIND; 3];

		let mut extension = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
			.binding_flags(&bind_flags)
			.build();

		let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
			.push_next(&mut extension)
			.bindings(&bindless_bindings)
			.flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL);
		let bindless_layout =
			unsafe { logical_device.create_descriptor_set_layout(&create_info, None)? };

		let pool_sizes = [
			vk::DescriptorPoolSize::builder()
				.ty(vk::DescriptorType::STORAGE_BUFFER)
				.descriptor_count(1)
				.build(),
			vk::DescriptorPoolSize::builder()
				.ty(vk::DescriptorType::SAMPLED_IMAGE)
				.descriptor_count(1)
				.build(),
			vk::DescriptorPoolSize::builder()
				.ty(vk::DescriptorType::SAMPLER)
				.descriptor_count(1)
				.build(),
		];

		let create_info = vk::DescriptorPoolCreateInfo::builder()
			.pool_sizes(&pool_sizes)
			.max_sets(1)
			.flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);
		let bindless_pool = unsafe { logical_device.create_descriptor_pool(&create_info, None)? };

		let layouts = [bindless_layout];

		let create_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(bindless_pool)
			.set_layouts(&layouts);
		let bindless_set = unsafe { logical_device.allocate_descriptor_sets(&create_info)? };

		let bindles_info = BindlessInfo {
			textures: Vec::new(),
			null_texture: None,

			buffers: Vec::new(),
			null_buffer: None,

			samplers: Vec::new(),
			null_sampler: None,
		};

		let result = Arc::new(Device {
			owner: instance,

			logical: logical_device,
			physical: physical_device,

			graphics_queue: graphics_queue.map(Mutex::new),
			presentation_queue: presentation_queue.map(Mutex::new),

			graphics_family_index,
			surface_family_index,

			work: Mutex::new(WorkContainer {
				last_id: 0,
				in_queue: HashMap::new(),
			}),

			surface,

			swapchain: Mutex::new(None),
			thread_info: Mutex::new(HashMap::new()),

			bindless_info: Mutex::new(bindles_info),

			bindless_layout,
			bindless_pool,
			bindless_set: bindless_set[0],

			render_passes: Mutex::new(Vec::with_capacity(128)),
		});

		{
			let mut swapchain = result.swapchain.lock().unwrap();
			*swapchain = Some(Swapchain::new(result.clone())?);
		}

		// Create null texture
		let null_texutre = Texture::new(
			result.clone(),
			MemoryType::DeviceLocal,
			TextureUsage::SAMPLED,
			Format::RGBA_U8,
			64,
			64,
			1,
		)
		.unwrap();

		// Create the null buffer
		let null_buffer = Buffer::new(
			result.clone(),
			BufferUsage::CONSTANTS,
			MemoryType::HostVisible,
			16,
		)
		.unwrap();

		let null_sampler = Sampler::new(result.clone(), Default::default()).unwrap();

		{
			let mut bindless = result.bindless_info.lock().unwrap();
			bindless.null_texture = Some(null_texutre);
			bindless.null_buffer = Some(null_buffer);
			bindless.null_sampler = Some(null_sampler);
		}

		Ok(result)
	}

	pub fn acquire_backbuffer(&self) -> Result<Arc<Texture>> {
		assert!(self.surface.is_some());

		let mut swapchain = self.swapchain.lock().unwrap();

		let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
		let semaphore = unsafe {
			self.logical
				.create_semaphore(&semaphore_create_info, None)?
		};

		let swapchain_khr = khr::Swapchain::new(&self.owner.instance, &self.logical);
		let (index, _) = unsafe {
			swapchain_khr.acquire_next_image(
				swapchain.as_ref().unwrap().handle,
				1 << 63,
				semaphore,
				vk::Fence::default(),
			)
		}?;

		swapchain.as_mut().unwrap().current = Some(index as usize);

		unsafe { self.logical.destroy_semaphore(semaphore, None) };

		Ok(swapchain.as_ref().unwrap().backbuffers[index as usize].clone())
	}

	pub fn submit_graphics(
		&self,
		command_buffers: Vec<GraphicsCommandBuffer>,
		wait_on: &[Receipt],
	) -> Receipt {
		self.update_bindless();

		let mut buffers = Vec::with_capacity(command_buffers.len());
		for it in command_buffers.iter() {
			buffers.push(it.command_buffer);
		}

		let semaphore_create_info = vk::SemaphoreCreateInfo::default();
		let semaphore = unsafe {
			self.logical
				.create_semaphore(&semaphore_create_info, None)
				.unwrap()
		};

		let fence_create_info = vk::FenceCreateInfo::builder();
		let fence = unsafe { self.logical.create_fence(&fence_create_info, None).unwrap() };

		let mut submit_info = vk::SubmitInfo::builder()
			.command_buffers(&buffers[..])
			.signal_semaphores(from_ref(&semaphore));

		unsafe {
			let queue = self.graphics_queue.as_ref().unwrap().lock().unwrap();

			self.remove_finished_work();

			let mut wait_semaphores = Vec::with_capacity(wait_on.len());
			let mut wait_stages = Vec::with_capacity(wait_on.len());
			if !wait_on.is_empty() {
				for it in wait_on.iter() {
					let sync = it.get();
					if sync.is_none() {
						continue;
					}
					let (semaphore, _) = sync.unwrap();

					wait_semaphores.push(semaphore);
					wait_stages.push(vk::PipelineStageFlags::BOTTOM_OF_PIPE);
				}

				submit_info = submit_info
					.wait_semaphores(&wait_semaphores[..])
					.wait_dst_stage_mask(&wait_stages[..]);
			}
			self.logical
				.queue_submit(*queue, from_ref(&submit_info), fence)
				.expect("Failed to submit graphics commands to gpu");
		}

		let owner = command_buffers[0].owner.clone();

		let id = self.push_work(WorkEntry {
			semaphore,
			fence,
			variant: WorkVariant::Graphics(command_buffers),
			thread_id: std::thread::current().id(),
		});
		Receipt::new(owner, id)
	}

	pub fn display(&self, wait_on: &[Receipt]) {
		assert!(self.surface.is_some());

		self.remove_finished_work();

		let mut swapchain = self.swapchain.lock().unwrap();
		let swapchain_khr = khr::Swapchain::new(&self.owner.instance, &self.logical);

		let index = swapchain
			.as_ref()
			.unwrap()
			.current
			.expect("Backbuffer was not acquired") as u32;

		let mut present_info = vk::PresentInfoKHR::builder()
			.swapchains(from_ref(&swapchain.as_ref().unwrap().handle))
			.image_indices(from_ref(&index));

		let mut wait_semaphores = Vec::with_capacity(wait_on.len());
		if !wait_on.is_empty() {
			for it in wait_on.iter() {
				let sync = it.get();
				if sync.is_none() {
					continue;
				}
				let (semaphore, _) = sync.unwrap();

				wait_semaphores.push(semaphore);
			}

			present_info = present_info.wait_semaphores(&wait_semaphores[..]);
		}

		let result = unsafe {
			let queue = self.presentation_queue.as_ref().unwrap().lock().unwrap();

			swapchain_khr.queue_present(*queue, &present_info)
		};
		if result.is_err() {
			*swapchain =
				Swapchain::new(swapchain.as_ref().unwrap().backbuffers[0].owner.clone()).ok();
		}
	}

	pub fn remove_finished_work(&self) {
		let mut work = self.work.lock().unwrap();

		work.in_queue.retain(|_, v| {
			if v.thread_id != std::thread::current().id() {
				return true;
			}

			unsafe {
				let result = self.logical.get_fence_status(v.fence).unwrap();
				if result {
					self.logical.destroy_fence(v.fence, None);
					self.logical.destroy_semaphore(v.semaphore, None);
				}
				!result
			}
		});
	}

	pub fn update_bindless(&self) {
		let bindless = self.bindless_info.lock().unwrap();

		let null_buffer = bindless.null_buffer.as_ref().unwrap();
		let mut buffers = Vec::with_capacity(bindless.buffers.len());
		for it in bindless.buffers.iter() {
			if it.strong_count() == 0 {
				let info = vk::DescriptorBufferInfo::builder()
					.buffer(null_buffer.handle)
					.range(null_buffer.size as u64)
					.build();

				buffers.push(info);
				continue;
			}

			let buffer = it.upgrade().unwrap();
			let info = vk::DescriptorBufferInfo::builder()
				.buffer(buffer.handle)
				.range(buffer.size as u64)
				.build();

			buffers.push(info);
		}

		let buffers_set_write = vk::WriteDescriptorSet::builder()
			.dst_set(self.bindless_set)
			.dst_binding(0)
			.descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
			.buffer_info(&buffers[..])
			.build();

		let null_texture = bindless.null_texture.as_ref().unwrap();
		let mut image_infos = Vec::with_capacity(bindless.textures.len()); // TODO: Use temp allocator
		for it in bindless.textures.iter() {
			if it.strong_count() == 0 {
				let image_info = vk::DescriptorImageInfo::builder()
					.image_view(null_texture.view)
					.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
					.build();

				image_infos.push(image_info);
				continue;
			}

			let tex = it.upgrade().unwrap();

			let image_info = vk::DescriptorImageInfo::builder()
				.image_view(tex.view)
				.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
				.build();

			image_infos.push(image_info);
		}

		let null_sampler = bindless.null_sampler.as_ref().unwrap();
		let mut sampler_infos = Vec::with_capacity(bindless.samplers.len()); // TODO: Use temp allocator
		for it in bindless.samplers.iter() {
			if it.strong_count() == 0 {
				let sampler_info = vk::DescriptorImageInfo::builder()
					.sampler(null_sampler.handle)
					.build();

				sampler_infos.push(sampler_info);
				continue;
			}

			let sampler = it.upgrade().unwrap();
			let sampler_info = vk::DescriptorImageInfo::builder()
				.sampler(sampler.handle)
				.build();

			sampler_infos.push(sampler_info);
		}

		let image_set_write = vk::WriteDescriptorSet::builder()
			.dst_set(self.bindless_set)
			.dst_binding(1)
			.descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
			.image_info(&image_infos[..])
			.build();

		let samplers_set_write = vk::WriteDescriptorSet::builder()
			.dst_set(self.bindless_set)
			.dst_binding(2)
			.descriptor_type(vk::DescriptorType::SAMPLER)
			.image_info(&sampler_infos[..])
			.build();

		let set_writes = [buffers_set_write, image_set_write, samplers_set_write];

		unsafe { self.logical.update_descriptor_sets(&set_writes, &[]) };
	}

	pub fn wait_for_idle(&self) {
		unsafe { self.logical.device_wait_idle().unwrap() };
	}
}
