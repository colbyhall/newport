use super::{
	vk_format,
	Device,
};
use crate::{
	Format,
	Result,
};

use ash::version::DeviceV1_0;
use ash::vk;

use std::slice::from_ref;
use std::sync::Arc;

pub struct RenderPass {
	pub handle: vk::RenderPass,

	pub attachments: Vec<Format>,
}

impl RenderPass {
	pub fn new(owner: &Device, attachment_values: Vec<Format>) -> Result<Arc<RenderPass>> {
		let mut color_refs = Vec::with_capacity(attachment_values.len());
		let mut depth_ref = None;

		let mut attachments = Vec::with_capacity(attachment_values.len());

		for (index, it) in attachment_values.iter().enumerate() {
			if it.is_depth() {
				let format = vk_format(*it);

				let attachment = vk::AttachmentDescription::builder()
					.format(format)
					.samples(vk::SampleCountFlags::TYPE_1)
					.load_op(vk::AttachmentLoadOp::DONT_CARE)
					.store_op(vk::AttachmentStoreOp::STORE)
					.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
					.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::UNDEFINED)
					.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
				attachments.push(attachment.build());

				let the_ref = vk::AttachmentReference::builder()
					.attachment((attachment_values.len() - 1) as u32)
					.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

				depth_ref = Some(the_ref.build())
			} else {
				let format = vk_format(*it);

				let attachment = vk::AttachmentDescription::builder()
					.format(format)
					.samples(vk::SampleCountFlags::TYPE_1)
					.load_op(vk::AttachmentLoadOp::DONT_CARE)
					.store_op(vk::AttachmentStoreOp::STORE)
					.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
					.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
					.initial_layout(vk::ImageLayout::UNDEFINED)
					.final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
				attachments.push(attachment.build());

				let the_ref = vk::AttachmentReference::builder()
					.attachment(index as u32)
					.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
				color_refs.push(the_ref.build());
			}
		}

		// Currently we're only going to support 1 subpass as no other API has subpasses
		let mut subpass = vk::SubpassDescription::builder()
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.color_attachments(&color_refs[..]);

		if let Some(depth) = &depth_ref {
			subpass = subpass.depth_stencil_attachment(depth);
		}

		let mut stage_mask = vk::PipelineStageFlags::empty();
		let mut access_mask = vk::AccessFlags::empty();

		if !color_refs.is_empty() {
			stage_mask |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
			access_mask |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
		}

		if depth_ref.is_some() {
			stage_mask |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
			access_mask |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
		}

		let dependency = vk::SubpassDependency::builder()
			.src_subpass(vk::SUBPASS_EXTERNAL)
			.src_stage_mask(stage_mask)
			.dst_stage_mask(stage_mask)
			.dst_access_mask(access_mask);

		let create_info = vk::RenderPassCreateInfo::builder()
			.attachments(&attachments[..])
			.subpasses(from_ref(&subpass))
			.dependencies(from_ref(&dependency));

		unsafe {
			let handle = owner.logical.create_render_pass(&create_info, None)?;

			Ok(Arc::new(RenderPass {
				handle,
				attachments: attachment_values,
			}))
		}
	}
}

impl Drop for RenderPass {
	fn drop(&mut self) {
		// todo!()
	}
}
