use crate::*;

use std::{
	collections::HashMap,
	sync::Arc,
};

use asset::{
	Asset,
	Importer,
};
use serde::{
	self as serde,
	ron,
	Deserialize,
	Serialize,
};

use bitflags::bitflags;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum DrawMode {
	Fill,
	Line,
	Point,
}

bitflags! {
	pub struct CullMode: u32 {
		const FRONT = 0b01;
		const BACK  = 0b10;
	}
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum CompareOp {
	Never,
	Less,           // A < B
	Equal,          // A == B
	LessOrEqual,    // A < B || A == B
	Greater,        // A > B
	NotEqual,       // A != B
	GreaterOrEqual, // A > B || A == B
	Always,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum BlendOp {
	Add,
	Subtract,
	ReverseSubtract,
	Min,
	Max,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum BlendFactor {
	Zero,
	One,

	SrcColor,
	OneMinusSrcColor,
	DstColor,
	OneMinusDstColor,

	SrcAlpha,
	OneMinusSrcAlpha,
}

bitflags! {
	pub struct ColorMask: u32 {
		const RED   = 0b0001;
		const GREEN = 0b0010;
		const BLUE  = 0b0100;
		const ALPHA = 0b1000;
	}
}

pub struct GraphicsPipelineDescription {
	pub attachments: Vec<Format>,

	pub shaders: Vec<Shader>,

	pub vertex_attributes: Vec<Constant>,

	pub draw_mode: DrawMode,
	pub line_width: f32,

	pub cull_mode: CullMode,
	pub color_mask: ColorMask,

	pub blend_enabled: bool,

	pub src_color_blend_factor: BlendFactor,
	pub dst_color_blend_factor: BlendFactor,
	pub color_blend_op: BlendOp,

	pub src_alpha_blend_factor: BlendFactor,
	pub dst_alpha_blend_factor: BlendFactor,
	pub alpha_blend_op: BlendOp,

	pub depth_test: bool,
	pub depth_write: bool,
	pub depth_compare: CompareOp,

	pub constants: HashMap<String, Vec<ConstantMember>>,
	pub resources: HashMap<String, Resource>,
}

impl GraphicsPipelineDescription {
	pub(crate) fn push_constant_size(&self) -> usize {
		self.constants.len() * 4 + self.resources.len() * 4
	}
}

pub struct GraphicsPipelineBuilder<'a> {
	description: GraphicsPipelineDescription,
	device: Option<&'a Device>,
}

impl<'a> GraphicsPipelineBuilder<'a> {
	pub fn attachments(mut self, attachments: &[Format]) -> Self {
		self.description.attachments = attachments.to_vec();
		self
	}

	pub fn shaders(mut self, shaders: &[Shader]) -> Self {
		self.description.shaders = shaders.to_owned().to_vec();
		self
	}

	pub fn vertex_attributes(mut self, vertex_attributes: &[Constant]) -> Self {
		self.description.vertex_attributes = vertex_attributes.to_owned().to_vec();
		self
	}

	pub fn draw_mode(mut self, draw_mode: DrawMode) -> Self {
		self.description.draw_mode = draw_mode;
		self
	}

	pub fn line_width(mut self, line_width: f32) -> Self {
		self.description.line_width = line_width;
		self
	}

	pub fn cull_mode(mut self, cull_mode: CullMode) -> Self {
		self.description.cull_mode = cull_mode;
		self
	}

	pub fn color_mask(mut self, color_mask: ColorMask) -> Self {
		self.description.color_mask = color_mask;
		self
	}

	pub fn blend_enabled(mut self, blend_enabled: bool) -> Self {
		self.description.blend_enabled = blend_enabled;
		self
	}

	pub fn src_color_blend_factor(mut self, src_color_blend_factor: BlendFactor) -> Self {
		self.description.src_color_blend_factor = src_color_blend_factor;
		self
	}

	pub fn dst_color_blend_factor(mut self, dst_color_blend_factor: BlendFactor) -> Self {
		self.description.dst_color_blend_factor = dst_color_blend_factor;
		self
	}

	pub fn color_blend_op(mut self, color_blend_op: BlendOp) -> Self {
		self.description.color_blend_op = color_blend_op;
		self
	}

	pub fn src_alpha_blend_factor(mut self, src_alpha_blend_factor: BlendFactor) -> Self {
		self.description.src_alpha_blend_factor = src_alpha_blend_factor;
		self
	}

	pub fn dst_alpha_blend_factor(mut self, dst_alpha_blend_factor: BlendFactor) -> Self {
		self.description.dst_alpha_blend_factor = dst_alpha_blend_factor;
		self
	}

	pub fn alpha_blend_op(mut self, alpha_blend_op: BlendOp) -> Self {
		self.description.alpha_blend_op = alpha_blend_op;
		self
	}

	pub fn depth_test(mut self, depth_test: bool) -> Self {
		self.description.depth_test = depth_test;
		self
	}

	pub fn depth_write(mut self, depth_write: bool) -> Self {
		self.description.depth_write = depth_write;
		self
	}

	pub fn depth_compare(mut self, depth_compare: CompareOp) -> Self {
		self.description.depth_compare = depth_compare;
		self
	}

	pub fn constant(mut self, name: impl ToString, members: Vec<ConstantMember>) -> Self {
		self.description.constants.insert(name.to_string(), members);
		self
	}

	pub fn constants(mut self, constants: HashMap<String, Vec<ConstantMember>>) -> Self {
		self.description.constants = constants;
		self
	}

	pub fn resource(mut self, name: impl ToString, resource: Resource) -> Self {
		self.description
			.resources
			.insert(name.to_string(), resource);
		self
	}

	pub fn resources(mut self, resources: HashMap<String, Resource>) -> Self {
		self.description.resources = resources;
		self
	}

	pub fn spawn(self) -> Result<GraphicsPipeline> {
		let device = match self.device {
			Some(device) => device,
			None => Gpu::device(),
		};

		Ok(GraphicsPipeline(api::GraphicsPipeline::new(
			device.0.clone(),
			self.description,
		)?))
	}
}

pub struct GraphicsPipeline(pub(crate) Arc<api::GraphicsPipeline>);

impl GraphicsPipeline {
	pub fn builder<'a>() -> GraphicsPipelineBuilder<'a> {
		GraphicsPipelineBuilder {
			description: GraphicsPipelineDescription {
				attachments: Default::default(),
				shaders: Default::default(),
				vertex_attributes: Default::default(),
				draw_mode: DrawMode::Fill,
				line_width: 1.0,

				cull_mode: CullMode::BACK,
				color_mask: ColorMask::all(),

				blend_enabled: false,

				src_color_blend_factor: BlendFactor::One,
				dst_color_blend_factor: BlendFactor::One,
				color_blend_op: BlendOp::Add,

				src_alpha_blend_factor: BlendFactor::One,
				dst_alpha_blend_factor: BlendFactor::One,
				alpha_blend_op: BlendOp::Add,

				depth_test: false,
				depth_write: false,
				depth_compare: CompareOp::Always,

				constants: Default::default(),
				resources: Default::default(),
			},
			device: None,
		}
	}
}

impl Asset for GraphicsPipeline {}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct DepthStencilStates {
	#[serde(default)]
	pub depth_test: bool,
	#[serde(default)]
	pub depth_write: bool,
	#[serde(default = "DepthStencilStates::default_depth_compare")]
	pub depth_compare: CompareOp,
	#[serde(default = "DepthStencilStates::default_depth_stencil_format")]
	pub depth_stencil_format: Format,
}

impl DepthStencilStates {
	fn default_depth_compare() -> CompareOp {
		CompareOp::Less
	}

	fn default_depth_stencil_format() -> Format {
		Format::D24_S8
	}
}

impl Default for DepthStencilStates {
	fn default() -> Self {
		Self {
			depth_test: false,
			depth_write: false,
			depth_compare: Self::default_depth_compare(),
			depth_stencil_format: Self::default_depth_stencil_format(),
		}
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "CullMode", crate = "self::serde")]
pub enum CullModeSerde {
	Front,
	Back,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "ColorMask", crate = "self::serde")]
pub enum ColorMaskSerde {
	Red,
	Green,
	Blue,
	Alpha,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct RenderStates {
	#[serde(default = "RenderStates::default_draw_mode")]
	pub draw_mode: DrawMode,

	#[serde(default = "RenderStates::default_line_width")]
	pub line_width: f32,

	#[serde(default)]
	pub cull_mode: Vec<CullModeSerde>,

	#[serde(default = "RenderStates::default_color_mask")]
	pub color_mask: Vec<ColorMaskSerde>,
}

impl RenderStates {
	fn default_draw_mode() -> DrawMode {
		DrawMode::Fill
	}

	fn default_line_width() -> f32 {
		1.0
	}

	fn default_color_mask() -> Vec<ColorMaskSerde> {
		vec![
			ColorMaskSerde::Red,
			ColorMaskSerde::Green,
			ColorMaskSerde::Blue,
			ColorMaskSerde::Alpha,
		]
	}
}

impl Default for RenderStates {
	fn default() -> Self {
		Self {
			draw_mode: Self::default_draw_mode(),
			line_width: Self::default_line_width(),
			cull_mode: Default::default(),
			color_mask: Self::default_color_mask(),
		}
	}
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "self::serde")]
pub struct BlendStates {
	#[serde(default = "BlendStates::default_blend_factor")]
	pub src_blend_factor: BlendFactor,

	#[serde(default = "BlendStates::default_blend_factor")]
	pub dst_blend_factor: BlendFactor,

	#[serde(default = "BlendStates::default_blend_op")]
	pub blend_op: BlendOp,
}

impl BlendStates {
	fn default_blend_factor() -> BlendFactor {
		BlendFactor::One
	}

	fn default_blend_op() -> BlendOp {
		BlendOp::Add
	}
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct ConstantMember(String, Constant);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(crate = "self::serde")]
pub enum Constant {
	Uint32,
	Int32,
	Float32,
	Vector2,
	Vector3,
	Vector4,
	Color,
	Matrix4,
}

impl Constant {
	fn into_type_string(self) -> &'static str {
		match self {
			Self::Uint32 => "uint",
			Self::Int32 => "int",
			Self::Float32 => "float",

			Self::Vector2 => "float2",
			Self::Vector3 => "float3",
			Self::Vector4 => "float4",
			Self::Color => "float4",

			Self::Matrix4 => "float4x4",
		}
	}

	pub fn size(self) -> usize {
		match self {
			Self::Uint32 | Self::Int32 | Self::Float32 => 4,
			Self::Vector2 => 8,
			Self::Vector3 => 12,
			Self::Vector4 | Self::Color => 16,

			Self::Matrix4 => 16 * 4,
		}
	}
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum Resource {
	Texture,
	Sampler(SamplerDescription),
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum SystemSemantics {
	VertexId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct VertexShader {
	#[serde(default)]
	pub attributes: Vec<ConstantMember>,
	#[serde(default)]
	pub system_semantics: Vec<SystemSemantics>,

	#[serde(default)]
	pub exports: Vec<ConstantMember>,
	pub code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct PixelShader {
	pub exports: Vec<(String, Format)>,
	pub code: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "GraphicsPipeline", crate = "self::serde")]
pub struct GraphicsPipelineFile {
	#[serde(default)]
	pub render_states: RenderStates,

	#[serde(default)]
	pub color_blend: Option<BlendStates>,

	#[serde(default)]
	pub alpha_blend: Option<BlendStates>,

	#[serde(default)]
	pub depth_stencil_states: DepthStencilStates,

	#[serde(default)]
	pub constants: HashMap<String, Vec<ConstantMember>>,

	#[serde(default)]
	pub resources: HashMap<String, Resource>,

	#[serde(default)]
	pub common: String,

	pub vertex_shader: VertexShader,
	pub pixel_shader: PixelShader,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub(crate) struct GraphicsPipelineImporter {}

static SHADER_HEADER: &str = "
    #define NULL 0
    ByteAddressBuffer _all_buffers[]  : register(t0);
    Texture2D         _all_textures[] : register(t1);
    SamplerState      _all_samplers[] : register(s2);

    ByteAddressBuffer index_buffers(uint index) {
        return _all_buffers[index];
    }

    Texture2D index_textures(uint index) {
        return _all_textures[index];
    }

    SamplerState index_samplers(uint index) {
        return _all_samplers[index];
    }
";

impl Importer for GraphicsPipelineImporter {
	type Target = GraphicsPipeline;

	fn import(&self, bytes: &[u8]) -> asset::Result<Self::Target> {
		let contents = std::str::from_utf8(bytes)?;
		let file = ron::from_str(contents)?;

		let GraphicsPipelineFile {
			render_states,

			color_blend,
			alpha_blend,

			depth_stencil_states,

			vertex_shader,
			pixel_shader,

			constants,
			resources,

			common,
		} = file;

		let header = {
			let mut result = SHADER_HEADER.to_string();
			result.reserve(4096);

			// TODO: Check if this should go after
			result.push('\n');
			result.push_str(&common);
			result.push_str("\n\n");

			// If we have imports then we need to fill out the constants and build boilerplate
			if !constants.is_empty() || !resources.is_empty() {
				// First thing to do is build the push constants structure
				result.push_str("struct PushConstants {\n");
				for (name, _) in constants.iter() {
					result.push_str("    uint ");
					result.push_str(name);
					result.push_str(";\n");
				}
				for (name, _) in resources.iter() {
					result.push_str("    uint ");
					result.push_str(name);
					result.push_str(";\n");
				}
				result.push_str("};\n");
				result.push_str("[[vk::push_constant]] PushConstants push_constants;\n\n");

				// Secondly we must now define structs for constants and accessor boilerplate
				for (name, constants) in constants.iter() {
					// Declare constants structure
					result.push_str("struct ");

					let name_capitalized = {
						let mut c = name.chars();
						match c.next() {
							None => String::new(),
							Some(f) => f.to_uppercase().chain(c).collect(),
						}
					};

					result.push_str(&name_capitalized);
					result.push_str(" {\n");

					// Run through every item in constants and decalre it
					for ConstantMember(name, variant) in constants.iter() {
						result.push_str("    ");
						result.push_str(variant.into_type_string());
						result.push(' ');
						result.push_str(name);
						result.push_str(";\n");
					}

					result.push_str("};\n\n");

					// Generate custom get method declaration
					result.push_str(&name_capitalized);
					result.push_str(" get_");
					result.push_str(name);
					result.push_str("() {\n");

					// Grab the data from the buffer
					result.push_str("ByteAddressBuffer buffer = index_buffers((push_constants.");
					result.push_str(name);
					result.push_str(" >> 16) & 0xffff);\n");

					result.push_str(&name_capitalized);
					result.push_str(" result = buffer.Load<");
					result.push_str(&name_capitalized);
					result.push_str(">((push_constants.");
					result.push_str(name);
					result.push_str(" & 0xffff) * sizeof(");
					result.push_str(&name_capitalized);
					result.push_str("));");

					// Transpose any matrices
					for ConstantMember(name, variant) in constants.iter() {
						if *variant == Constant::Matrix4 {
							result.push_str("result.");
							result.push_str(name);
							result.push_str(" = transpose(result.");
							result.push_str(name);
							result.push_str(");\n");
						}
					}

					result.push_str("return result;\n}\n\n");
				}

				// Generate resource load boilerplate
				for (name, resource) in resources.iter() {
					// Generate custom load method declaration
					let resource_type = match resource {
						Resource::Texture => "Texture2D",
						Resource::Sampler { .. } => "SamplerState",
					};

					result.push_str(resource_type);
					result.push_str(" load_");
					result.push_str(name);
					result.push_str("() {\n");

					match resource {
						Resource::Texture => {
							result.push_str("return index_textures(push_constants.");
							result.push_str(name);
							result.push_str(");")
						}
						Resource::Sampler { .. } => {
							result.push_str("return index_samplers(push_constants.");
							result.push_str(name);
							result.push_str(");\n")
						}
					}

					result.push_str("}\n\n");
				}
			}

			result
		};

		let mut attachments: Vec<Format> = pixel_shader
			.exports
			.iter()
			.map(|(_, format)| *format)
			.collect();

		let vertex_attributes: Vec<Constant> = vertex_shader
			.attributes
			.iter()
			.map(|ConstantMember(_, variant)| *variant)
			.collect();

		let blend_enabled = color_blend.is_some() || alpha_blend.is_some();

		let color_blend = color_blend.unwrap_or(BlendStates {
			src_blend_factor: BlendFactor::One,
			dst_blend_factor: BlendFactor::One,
			blend_op: BlendOp::Add,
		});

		let alpha_blend = alpha_blend.unwrap_or(BlendStates {
			src_blend_factor: BlendFactor::One,
			dst_blend_factor: BlendFactor::One,
			blend_op: BlendOp::Add,
		});

		// Generate the pixel shader first to have access to exports
		let pixel_shader = {
			let PixelShader { exports, code } = pixel_shader;

			let imports = &vertex_shader.exports;

			// Start off with header
			let mut source = header.clone();

			source.push_str("struct PixelOutput {\n");
			for (index, (name, format)) in exports.iter().enumerate() {
				let mut name_uppercase = name.clone();
				name_uppercase.make_ascii_uppercase();

				let variant = match format {
					Format::BGR_U8_SRGB
					| Format::RGBA_F16
					| Format::RGB_U8
					| Format::RGB_U8_SRGB
					| Format::RGBA_U8
					| Format::RGBA_U8_SRGB => "float4",
					_ => unreachable!(),
				};

				let line = format!("    {} {} : SV_TARGET{};\n", variant, name, index);
				source.push_str(&line);
			}
			source.push_str("};\n\n");

			// Generate PixelInput based off of imports
			if !imports.is_empty() {
				source.push_str("struct PixelInput {\n");
				for ConstantMember(name, variant) in imports.iter() {
					let mut name_uppercase = name.clone();
					name_uppercase.make_ascii_uppercase();

					let line = format!(
						"    {} {} : {};\n",
						variant.into_type_string(),
						name,
						name_uppercase
					);
					source.push_str(&line);
				}
				source.push_str("};\n\n");

				source.push_str("PixelOutput main( PixelInput input ) {\n");
			} else {
				source.push_str("PixelOutput main( ) {\n");
			}

			source.push_str("PixelOutput output;");

			source.push_str(&code);
			source.push_str("\n}\n");

			// Compile to binary and then pass to device
			let binary =
				shader::compile("pixel.hlsl", &source, "main", ShaderVariant::Pixel).unwrap();

			Shader::builder(&binary, ShaderVariant::Pixel).spawn()?
		};

		// Generate the vertex shader
		let vertex_shader = {
			let VertexShader {
				attributes,
				system_semantics,
				exports,
				code,
			} = vertex_shader;

			// Start off with header
			let mut source = header;

			// Generate VertexOutput always. There will always be position
			source.push_str("struct VertexOutput {\n");
			for ConstantMember(name, variant) in exports.iter() {
				let mut name_uppercase = name.clone();
				name_uppercase.make_ascii_uppercase();

				let line = format!(
					"    {} {} : {};\n",
					variant.into_type_string(),
					name,
					name_uppercase
				);
				source.push_str(&line);
			}
			source.push_str("float4 position : SV_POSITION;\n");
			source.push_str("};\n\n");

			// Generate the VertexInput based off of attributes
			if !attributes.is_empty() || !system_semantics.is_empty() {
				source.push_str("struct VertexInput {\n");
				for ConstantMember(name, variant) in attributes.iter() {
					let mut name_uppercase = name.clone();
					name_uppercase.make_ascii_uppercase();

					let line = format!(
						"    {} {} : {};\n",
						variant.into_type_string(),
						name,
						name_uppercase
					);
					source.push_str(&line);
				}

				for semantic in system_semantics.iter() {
					let line = match semantic {
						SystemSemantics::VertexId => "uint vertex_id : SV_VertexID;\n",
					};
					source.push_str(line);
				}
				source.push_str("};\n\n");

				source.push_str("VertexOutput main( VertexInput input ) {\n");
				source.push_str("VertexOutput output;");
			} else {
				source.push_str("VertexOutput main( ) {\n");
			}
			source.push_str(&code);
			source.push_str("\n}\n");

			// Compile to binary and then pass to device
			let binary =
				shader::compile("vertex.hlsl", &source, "main", ShaderVariant::Vertex).unwrap();

			Shader::builder(&binary, ShaderVariant::Vertex).spawn()?
		};
		let shaders = vec![pixel_shader, vertex_shader];

		let (draw_mode, line_width, cull_mode, color_mask) = {
			let mut cull_mode = CullMode::empty();
			for it in render_states.cull_mode.iter() {
				match it {
					CullModeSerde::Front => cull_mode.insert(CullMode::FRONT),
					CullModeSerde::Back => cull_mode.insert(CullMode::BACK),
				}
			}

			let mut color_mask = ColorMask::empty();
			for it in render_states.color_mask.iter() {
				match it {
					ColorMaskSerde::Red => color_mask.insert(ColorMask::RED),
					ColorMaskSerde::Green => color_mask.insert(ColorMask::GREEN),
					ColorMaskSerde::Blue => color_mask.insert(ColorMask::BLUE),
					ColorMaskSerde::Alpha => color_mask.insert(ColorMask::ALPHA),
				}
			}

			(
				render_states.draw_mode,
				render_states.line_width,
				cull_mode,
				color_mask,
			)
		};

		if depth_stencil_states.depth_test {
			attachments.push(depth_stencil_states.depth_stencil_format);
		}

		GraphicsPipeline::builder()
			.attachments(&attachments)
			.shaders(&shaders)
			.vertex_attributes(&vertex_attributes)
			.draw_mode(draw_mode)
			.line_width(line_width)
			.cull_mode(cull_mode)
			.color_mask(color_mask)
			.blend_enabled(blend_enabled)
			.src_color_blend_factor(color_blend.src_blend_factor)
			.dst_color_blend_factor(color_blend.dst_blend_factor)
			.color_blend_op(color_blend.blend_op)
			.src_alpha_blend_factor(alpha_blend.src_blend_factor)
			.dst_alpha_blend_factor(alpha_blend.dst_blend_factor)
			.alpha_blend_op(alpha_blend.blend_op)
			.depth_test(depth_stencil_states.depth_test)
			.depth_write(depth_stencil_states.depth_write)
			.depth_compare(depth_stencil_states.depth_compare)
			.constants(constants)
			.resources(resources)
			.spawn()
			.map_err(|err| -> Box<dyn std::error::Error + 'static> { Box::new(err) })
	}
}
