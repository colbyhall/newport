use crate::*;

use std::{
	collections::HashMap,
	sync::Arc,
};

use asset::{
	Asset,
	Importer,
};
use engine::Engine;

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
	pub color_attachments: Vec<Format>,
	pub depth_attachment: Option<Format>,

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

pub struct GraphicsPipeline(pub(crate) Arc<api::GraphicsPipeline>);

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
}

impl DepthStencilStates {
	fn default_depth_compare() -> CompareOp {
		CompareOp::Less
	}
}

impl Default for DepthStencilStates {
	fn default() -> Self {
		Self {
			depth_test: false,
			depth_write: false,
			depth_compare: Self::default_depth_compare(),
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
		let engine = Engine::as_ref();
		let gpu: &Gpu = engine.module().ok_or(asset::AssetRefError::NoManager)?;
		let device = gpu.device();

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
			result.push_str("\n");
			result.push_str(&common);
			result.push_str("\n\n");

			// If we have imports then we need to fill out the constants and build boilerplate
			if constants.len() > 0 || resources.len() > 0 {
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
						result.push_str(" ");
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
					result.push_str(">(push_constants.");
					result.push_str(name);
					result.push_str(" & 0xffff);\n\n");

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

		let color_attachments = pixel_shader
			.exports
			.iter()
			.map(|(_, format)| *format)
			.collect();

		let vertex_attributes = vertex_shader
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
			if imports.len() > 0 {
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

			let main = "main";

			// Compile to binary and then pass to device
			let binary =
				shader::compile("pixel.hlsl", &source, main, ShaderVariant::Pixel).unwrap();
			let shader = device
				.create_shader(&binary[..], ShaderVariant::Pixel, main)
				.unwrap();
			shader
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
			let mut source = header.clone();

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
			if attributes.len() > 0 || system_semantics.len() > 0 {
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

			let main = "main";

			// Compile to binary and then pass to device
			let binary =
				shader::compile("vertex.hlsl", &source, main, ShaderVariant::Vertex).unwrap();

			let shader = device
				.create_shader(&binary[..], ShaderVariant::Vertex, main)
				.unwrap();

			shader
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

		let description = GraphicsPipelineDescription {
			color_attachments,
			depth_attachment: None, // TODO: Depth attachment,

			shaders,

			vertex_attributes,

			draw_mode,
			line_width,

			cull_mode,
			color_mask,

			blend_enabled,

			src_color_blend_factor: color_blend.src_blend_factor,
			dst_color_blend_factor: color_blend.dst_blend_factor,
			color_blend_op: color_blend.blend_op,

			src_alpha_blend_factor: alpha_blend.src_blend_factor,
			dst_alpha_blend_factor: alpha_blend.dst_blend_factor,
			alpha_blend_op: alpha_blend.blend_op,

			depth_test: depth_stencil_states.depth_test,
			depth_write: depth_stencil_states.depth_write,
			depth_compare: depth_stencil_states.depth_compare,

			constants,
			resources,
		};

		Ok(device.create_graphics_pipeline(description).unwrap())
	}
}

// impl Asset for GraphicsPipeline {
// 	fn load(bytes: &[u8], _path: &Path) -> (UUID, Self) {
// 		let engine = Engine::as_ref();
// 		let gpu: &Gpu = engine.module().unwrap();
// 		let device = gpu.device();

// 		let (id, pipeline_file): (UUID, GraphicsPipelineFile) = deserialize(bytes).unwrap();

// 	}
// }
