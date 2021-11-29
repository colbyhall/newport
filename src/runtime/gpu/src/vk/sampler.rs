use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::vk;

use crate::{
	Filter,
	SamplerDescription,
	Wrap,
};

use super::Device;

pub struct Sampler {
	pub owner: Arc<Device>,

	pub handle: vk::Sampler,

	// Index into the devices bindless array
	pub bindless: u32,
}

impl Sampler {
	pub fn new(owner: Arc<Device>, description: SamplerDescription) -> Result<Arc<Self>, ()> {
		fn filter_to_vk(filter: Filter) -> vk::Filter {
			match filter {
				Filter::Nearest => vk::Filter::NEAREST,
				Filter::Linear => vk::Filter::LINEAR,
			}
		}

		fn wrap_to_vk(wrap: Wrap) -> vk::SamplerAddressMode {
			match wrap {
				Wrap::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE,
				Wrap::Repeat => vk::SamplerAddressMode::REPEAT,
			}
		}

		let create_info = vk::SamplerCreateInfo::builder()
			.min_filter(filter_to_vk(description.min_filter))
			.mag_filter(filter_to_vk(description.mag_filter))
			.address_mode_u(wrap_to_vk(description.address_u))
			.address_mode_v(wrap_to_vk(description.address_v))
			.address_mode_w(wrap_to_vk(description.address_w))
			.border_color(vk::BorderColor::INT_OPAQUE_BLACK);

		let sampler = unsafe { owner.logical.create_sampler(&create_info, None) };
		if sampler.is_err() {
			return Err(());
		}
		let sampler = sampler.unwrap();

		let mut bindless = owner.bindless_info.lock().unwrap();

		let found = bindless
			.samplers
			.iter_mut()
			.enumerate()
			.find(|(_, x)| x.strong_count() == 0)
			.map(|(index, _)| index);

		let index = found.unwrap_or(bindless.samplers.len());

		let result = Arc::new(Sampler {
			owner: owner.clone(),
			handle: sampler,
			bindless: index as u32,
		});

		let weak = Arc::downgrade(&result);
		if found.is_some() {
			bindless.samplers[index] = weak;
		} else {
			bindless.samplers.push(weak);
		}

		Ok(result)
	}
}
