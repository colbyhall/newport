use crate::{ Format, };
use super::{ Device, vk_format };

use ash::vk;
use ash::version::DeviceV1_0;

use std::sync::Arc;
use std::slice::from_ref;

pub struct RenderPass {
    pub owner: Arc<Device>,

    pub handle: vk::RenderPass,

    pub colors: Vec<Format>,
    pub depth:  Option<Format>
}

impl RenderPass {
    pub fn new(owner: Arc<Device>, colors: Vec<Format>, depth: Option<Format>) -> Result<Arc<RenderPass>, ()> {
        let mut color_refs = Vec::with_capacity(colors.len());
        
        let num_attachments = {
            let num = colors.len();
            if depth.is_some() {
                num + 1
            } else {
                num
            }
        };

        let mut attachments = Vec::with_capacity(num_attachments);

        for (index, it) in colors.iter().enumerate() {
            let format = vk_format(*it);

            let attachment = vk::AttachmentDescription::builder()
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
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

        // Currently we're only going to support 1 subpass as no other API has subpasses
        let mut subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs[..]);

        let depth_refs = if depth.is_some() {
            let depth = depth.unwrap();
            let format = vk_format(depth);

            let attachment = vk::AttachmentDescription::builder()
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            attachments.push(attachment.build());
            
            let the_ref = vk::AttachmentReference::builder()
                .attachment(num_attachments as u32)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
                
            Some(the_ref.build())
        } else {
            None
        };

        if depth_refs.is_some() {
            subpass = subpass.depth_stencil_attachment(depth_refs.as_ref().unwrap());
        }

        let mut stage_mask = vk::PipelineStageFlags::empty();
        let mut access_mask = vk::AccessFlags::empty();

        if colors.len() > 0 {
            stage_mask |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
            access_mask |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        }

        if depth.is_some() {
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
            let handle = owner.logical.create_render_pass(&create_info, None);
            if handle.is_err() {
                return Err(());
            }
            let handle = handle.unwrap();

            Ok(Arc::new(RenderPass{
                owner: owner,
                handle: handle,
                colors: colors,
                depth:  depth,
            }))
        }
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        todo!()
    }
}