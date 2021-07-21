use super::Device;
use super::Sampler;
use crate::Constant;
use crate::{
	BlendFactor,
	BlendOp,
	ColorMask,
	CullMode,
	DrawMode,
	GraphicsPipelineDescription,
	Resource,
	ShaderVariant,
};

use ash::version::DeviceV1_0;
use ash::vk;

use std::ffi::CString;
use std::slice::from_ref;
use std::sync::Arc;

fn shader_variant_to_shader_stage(variant: ShaderVariant) -> vk::ShaderStageFlags {
	match variant {
		ShaderVariant::Vertex => vk::ShaderStageFlags::VERTEX,
		ShaderVariant::Pixel => vk::ShaderStageFlags::FRAGMENT,
	}
}

pub struct GraphicsPipeline {
	pub owner: Arc<Device>,

	pub handle: vk::Pipeline,
	pub layout: vk::PipelineLayout,

	pub description: GraphicsPipelineDescription,

	// Store samplers with an index which is the push constant index
	pub samplers: Vec<(Arc<Sampler>, usize)>,
}

impl GraphicsPipeline {
	pub fn new(
		owner: Arc<Device>,
		description: GraphicsPipelineDescription,
	) -> Result<Arc<GraphicsPipeline>, ()> {
		assert!(description.shaders.len() > 0);

		// Create all the shader staage info for pipeline
		let mut shader_stages = Vec::with_capacity(description.shaders.len());
		for it in description.shaders.iter() {
			let stage = shader_variant_to_shader_stage(it.0.variant);

			let main = CString::new(it.0.main.clone()).unwrap();

			let stage_info = vk::PipelineShaderStageCreateInfo::builder()
				.stage(stage)
				.module(it.0.module)
				.name(&main)
				.build();

			main.into_raw(); // LEAK LEAK LEAK

			shader_stages.push(stage_info);
		}

		let mut stride = 0;
		for it in description.vertex_attributes.iter() {
			stride += it.size();
		}

		// Setup the vertex attributes
		let binding = vk::VertexInputBindingDescription::builder()
			.binding(0)
			.stride(stride as u32)
			.input_rate(vk::VertexInputRate::VERTEX);

		let mut attributes = Vec::with_capacity(description.vertex_attributes.len());
		let mut offset = 0;
		for (index, it) in description.vertex_attributes.iter().enumerate() {
			let format = match it {
				Constant::Int32 => vk::Format::R32_SINT,
				Constant::Uint32 => vk::Format::R32_UINT,
				Constant::Float32 => vk::Format::R32_SFLOAT,
				Constant::Vector2 => vk::Format::R32G32_SFLOAT,
				Constant::Vector3 => vk::Format::R32G32B32_SFLOAT,
				Constant::Vector4 => vk::Format::R32G32B32A32_SFLOAT,
				Constant::Color => vk::Format::R32G32B32A32_SFLOAT,
				_ => todo!(),
			};

			let attr = vk::VertexInputAttributeDescription::builder()
				.binding(0)
				.location(index as u32)
				.offset(offset as u32)
				.format(format);

			// TODO: Do this properly. This currently just uses the size of offsets but this doesnt count for alignment
			offset += it.size();

			attributes.push(attr.build());
		}

		let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
			.vertex_binding_descriptions(from_ref(&binding))
			.vertex_attribute_descriptions(&attributes[..]);

		let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST);

		let viewport = vk::Viewport::builder()
			.width(100.0)
			.height(100.0)
			.max_depth(1.0);
		let scissor =
			vk::Rect2D::builder().extent(vk::Extent2D::builder().width(100).height(100).build());

		let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
			.viewports(from_ref(&viewport))
			.scissors(from_ref(&scissor));

		let polygon_mode = match description.draw_mode {
			DrawMode::Fill => vk::PolygonMode::FILL,
			DrawMode::Line => vk::PolygonMode::LINE,
			DrawMode::Point => vk::PolygonMode::POINT,
		};

		let mut cull = vk::CullModeFlags::NONE;
		if description.cull_mode.contains(CullMode::FRONT) {
			cull |= vk::CullModeFlags::FRONT;
		}
		if description.cull_mode.contains(CullMode::BACK) {
			cull |= vk::CullModeFlags::BACK;
		}

		// NOTE: Depth Testing goes around here somewhere
		let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::builder()
			.polygon_mode(polygon_mode)
			.cull_mode(cull)
			.front_face(vk::FrontFace::CLOCKWISE)
			.line_width(description.line_width);

		let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1)
			.min_sample_shading(1.0);

		// Setting up blending and converting data types
		fn blend_factor(fc: BlendFactor) -> vk::BlendFactor {
			match fc {
				BlendFactor::Zero => return vk::BlendFactor::ZERO,
				BlendFactor::One => return vk::BlendFactor::ONE,
				BlendFactor::SrcColor => return vk::BlendFactor::SRC_COLOR,
				BlendFactor::OneMinusSrcColor => return vk::BlendFactor::ONE_MINUS_SRC_COLOR,
				BlendFactor::DstColor => return vk::BlendFactor::DST_COLOR,
				BlendFactor::OneMinusDstColor => return vk::BlendFactor::ONE_MINUS_DST_COLOR,
				BlendFactor::SrcAlpha => return vk::BlendFactor::SRC_ALPHA,
				BlendFactor::OneMinusSrcAlpha => return vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
			}
		}

		fn blend_op(a: BlendOp) -> vk::BlendOp {
			match a {
				BlendOp::Add => vk::BlendOp::ADD,
				BlendOp::Subtract => vk::BlendOp::SUBTRACT,
				BlendOp::ReverseSubtract => vk::BlendOp::REVERSE_SUBTRACT,
				BlendOp::Min => vk::BlendOp::MIN,
				BlendOp::Max => vk::BlendOp::MAX,
			}
		}

		let mut color_write_mask = vk::ColorComponentFlags::default();
		if description.color_mask.contains(ColorMask::RED) {
			color_write_mask |= vk::ColorComponentFlags::R;
		}
		if description.color_mask.contains(ColorMask::GREEN) {
			color_write_mask |= vk::ColorComponentFlags::G;
		}
		if description.color_mask.contains(ColorMask::BLUE) {
			color_write_mask |= vk::ColorComponentFlags::B;
		}
		if description.color_mask.contains(ColorMask::ALPHA) {
			color_write_mask |= vk::ColorComponentFlags::A;
		}

		let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
			.blend_enable(description.blend_enabled)
			.src_color_blend_factor(blend_factor(description.src_color_blend_factor))
			.dst_color_blend_factor(blend_factor(description.dst_color_blend_factor))
			.color_blend_op(blend_op(description.color_blend_op))
			.src_alpha_blend_factor(blend_factor(description.src_alpha_blend_factor))
			.dst_alpha_blend_factor(blend_factor(description.dst_alpha_blend_factor))
			.alpha_blend_op(blend_op(description.alpha_blend_op))
			.color_write_mask(color_write_mask)
			.build();

		let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
			.logic_op(vk::LogicOp::COPY)
			.attachments(from_ref(&color_blend_attachment));

		let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

		let dynamic_state =
			vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

		let layouts = [owner.bindless_layout];
		let mut pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder() // TODO: Do bindless descriptor layout
			.set_layouts(&layouts);

		let push_constant_size = description.push_constant_size();
		assert!(push_constant_size <= 128); // Min push contsant size

		let range = vk::PushConstantRange::builder()
			.size(push_constant_size as u32)
			.stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS);

		if push_constant_size > 0 {
			pipeline_layout_info = pipeline_layout_info.push_constant_ranges(from_ref(&range));
		}

		let layout = unsafe {
			owner
				.logical
				.create_pipeline_layout(&pipeline_layout_info, None)
		};
		if layout.is_err() {
			return Err(());
		}
		let layout = layout.unwrap();

		let render_pass = super::RenderPass::new(
			owner.clone(),
			description.color_attachments.clone(),
			description.depth_attachment,
		)?;

		let create_info = vk::GraphicsPipelineCreateInfo::builder()
			.stages(&shader_stages[..])
			.vertex_input_state(&vertex_input_state)
			.input_assembly_state(&input_assembly_state)
			.viewport_state(&viewport_state)
			.rasterization_state(&rasterizer_state)
			.multisample_state(&multisample_state)
			.color_blend_state(&color_blend_state)
			.dynamic_state(&dynamic_state)
			.layout(layout)
			.render_pass(render_pass.handle)
			.base_pipeline_index(-1);

		let handle = unsafe {
			owner.logical.create_graphics_pipelines(
				vk::PipelineCache::default(),
				from_ref(&create_info),
				None,
			)
		};
		if handle.is_err() {
			return Err(());
		}
		let handle = handle.unwrap();

		// Create samplers based off resources
		// TODO: Have some central hash map of samplers so we don't have so many
		let samplers = description
			.resources
			.iter()
			.enumerate()
			.filter(|(_, (_, resource))| match resource {
				Resource::Sampler(_) => true,
				_ => false,
			})
			.map(|(index, (_, resource))| match resource {
				Resource::Sampler(description) => {
					let sampler = Sampler::new(owner.clone(), *description).unwrap();
					(sampler, index)
				}
				_ => unreachable!(),
			})
			.collect();

		Ok(Arc::new(GraphicsPipeline {
			owner,

			handle: handle[0],
			layout,

			description,
			samplers,
		}))
	}
}