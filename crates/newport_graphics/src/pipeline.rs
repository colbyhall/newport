use crate::{
    gpu,
    asset,
    engine,
    serde,

    Graphics,
};

use asset::{
    Asset,
    UUID,
    deserialize,
};

use serde::{
    Serialize,
    Deserialize
};

use engine::{
    Engine,
};

use std::path::Path;

#[derive(Serialize, Deserialize)]
#[serde(rename = "Pipeline", crate = "self::serde")]
pub struct PipelineFile {
    #[serde(default)]
    render_states: RenderStates,
    
    #[serde(default)]
    color_blend: Option<BlendStates>,

    #[serde(default)]
    alpha_blend: Option<BlendStates>,

    #[serde(default)]
    depth_stencil_states: DepthStencilStates,
    
    #[serde(default)]
    imports: Vec<Item>,

    #[serde(default)]
    common: String,

    vertex_shader: Option<VertexShader>,
    pixel_shader:  Option<PixelShader>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct DepthStencilStates {
    #[serde(default)]
    depth_test:    bool,
    #[serde(default)]
    depth_write:   bool,
    #[serde(default = "DepthStencilStates::default_depth_compare")]
    depth_compare: gpu::CompareOp,
}

impl DepthStencilStates {
    fn default_depth_compare() -> gpu::CompareOp {
        gpu::CompareOp::Less
    }
}

impl Default for DepthStencilStates {
    fn default() -> Self {
        Self{
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
    Back
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "ColorMask", crate = "self::serde")]
pub enum ColorMaskSerde {
    Red,
    Green,
    Blue,
    Alpha
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct RenderStates {
    #[serde(default = "RenderStates::default_draw_mode")]
    draw_mode:  gpu::DrawMode,

    #[serde(default = "RenderStates::default_line_width")]
    line_width: f32,

    #[serde(default)]
    cull_mode:  Vec<CullModeSerde>,

    #[serde(default = "RenderStates::default_color_mask")]
    color_mask: Vec<ColorMaskSerde>,
}

impl RenderStates {
    fn default_draw_mode() -> gpu::DrawMode {
        gpu::DrawMode::Fill
    }

    fn default_line_width() -> f32 {
        1.0
    }

    fn default_color_mask() -> Vec<ColorMaskSerde> {
        vec![ColorMaskSerde::Red, ColorMaskSerde::Green, ColorMaskSerde::Blue, ColorMaskSerde::Alpha]
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
    src_blend_factor: gpu::BlendFactor,

    #[serde(default = "BlendStates::default_blend_factor")]
    dst_blend_factor: gpu::BlendFactor,

    #[serde(default = "BlendStates::default_blend_op")]
    blend_op:         gpu::BlendOp,
}

impl BlendStates {
    fn default_blend_factor() -> gpu::BlendFactor {
        gpu::BlendFactor::One
    }

    fn default_blend_op() -> gpu::BlendOp {
        gpu::BlendOp::Add
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct Item(String, ItemVariant);

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(crate = "self::serde")]
pub enum ItemVariant {
    Float32,
    Vector2,
    Vector3,
    Vector4,

    Matrix4,

    Texture,
}

impl ItemVariant {
    fn into_type_string(self) -> &'static str {
        match self {
            Self::Float32 => "float",
            
            Self::Vector2 => "float2",
            Self::Vector3 => "float3",
            Self::Vector4 => "float4",

            Self::Matrix4 => "float4x4",

            Self::Texture => "uint",
        }
    }
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
    pub attributes: Vec<Item>,
    #[serde(default)]
    pub system_semantics: Vec<SystemSemantics>,

    #[serde(default)]
    pub exports:    Vec<Item>,
    pub code:       String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct PixelShader {
    pub exports: Vec<(String, gpu::Format)>,
    pub code:    String,
}

pub struct Pipeline {
    pub file: PipelineFile,

    pub gpu: gpu::Pipeline,
}

static SHADER_HEADER: &str = "
    #define NULL 0
    ByteAddressBuffer _all_buffers[]  : register(t0);
    Texture2D         _all_textures[] : register(t1);
    SamplerState      _all_samplers[] : register(s2);

    struct Constants {
        uint index;
    };
    [[vk::push_constant]] Constants constants;

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

impl Asset for Pipeline {
    fn load(bytes: &[u8], _path: &Path) -> (UUID, Self) {
        let engine = Engine::as_ref();
        let graphics: &Graphics = engine.module().unwrap();
        let device = graphics.device();

        let (id, pipeline_file): (UUID, PipelineFile) = deserialize(bytes).unwrap();

        let PipelineFile {
            render_states,
            
            color_blend,
            alpha_blend,

            depth_stencil_states,

            vertex_shader,
            pixel_shader,
            
            imports,
            common

        } = &pipeline_file;

        let pixel_shader = pixel_shader.as_ref().expect("Pixel Shader is required until compute shader is implemented");
        let vertex_shader = vertex_shader.as_ref().expect("Vertex Shader is required until compute shader is implemented");

        let header = {
            let mut result = SHADER_HEADER.to_string();

            result.push_str("\n");
            result.push_str(&common);
            result.push_str("\n\n");

            if imports.len() > 0 {

                // Build GlobalImports which we'll pull from byte address buffer into
                result.push_str("struct Imports {\n");
                for Item(name, variant) in imports.iter() {
                    let line= format!(
                        "    {} {};\n", 
                        variant.into_type_string(), 
                        name,
                    );
                    result.push_str(&line);
                }
                result.push_str("};\n\n");

                result.push_str("Imports get_imports() {\n");
                result.push_str("Imports result = index_buffers(constants.index).Load<Imports>(0);\n");

                for Item(name, variant) in imports.iter() {
                    if *variant == ItemVariant::Matrix4 {
                        let line= format!(
                            "result.{} = transpose(result.{});\n", 
                            name,
                            name, 
                        );
                        result.push_str(&line);
                    }
                }

                result.push_str("return result;\n");
                result.push_str("}\n\n");

            }

            result
        };

        let colors = pixel_shader.exports.iter().map(|(_, format)|*format).collect();

        let render_pass = device.create_render_pass(colors, None).unwrap();

        let vertex_attributes = vertex_shader.attributes.iter().map(|Item(_, variant)| {
            match variant {
                ItemVariant::Float32 => gpu::VertexAttribute::Float32,
                ItemVariant::Vector2 => gpu::VertexAttribute::Vector2,
                ItemVariant::Vector3 => gpu::VertexAttribute::Vector3,
                ItemVariant::Vector4 => gpu::VertexAttribute::Vector4,

                ItemVariant::Texture => gpu::VertexAttribute::Uint32,

                _ => unreachable!(),
            }
        }).collect();

        let blend_enabled = color_blend.is_some() || alpha_blend.is_some();

        let color_blend = color_blend.unwrap_or(BlendStates{
            src_blend_factor: gpu::BlendFactor::One,
            dst_blend_factor: gpu::BlendFactor::One,
            blend_op: gpu::BlendOp::Add
        });

        let alpha_blend = alpha_blend.unwrap_or(BlendStates{
            src_blend_factor: gpu::BlendFactor::One,
            dst_blend_factor: gpu::BlendFactor::One,
            blend_op: gpu::BlendOp::Add
        });

        // Generate the pixel shader first to have access to exports
        let pixel_shader = {
            let PixelShader {
                exports,
                code,
            } = pixel_shader;

            let imports = &vertex_shader.exports;
            
            // Start off with header
            let mut source = header.clone();

            source.push_str("struct PixelOutput {\n");
            for (index, (name, format)) in exports.iter().enumerate() {
                let mut name_uppercase = name.clone();
                name_uppercase.make_ascii_uppercase();

                let variant = match format {
                        gpu::Format::BGR_U8_SRGB|gpu::Format::RGBA_F16|gpu::Format::RGB_U8|gpu::Format::RGB_U8_SRGB|gpu::Format::RGBA_U8|gpu::Format::RGBA_U8_SRGB => "float4",
                        _ => unreachable!(),
                    };

                let line= format!(
                    "    {} {} : SV_TARGET{};\n", 
                    variant, 
                    name, 
                    index
                );
                source.push_str(&line);
            }
            source.push_str("};\n\n");

            // Generate PixelInput based off of imports
            if imports.len() > 0 {
                source.push_str("struct PixelInput {\n");
                for Item(name, variant) in imports.iter() {
                    let mut name_uppercase = name.clone();
                    name_uppercase.make_ascii_uppercase();
    
                    let line= format!(
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
            let binary = gpu::shaders::compile("pixel.hlsl", &source, "main", gpu::ShaderVariant::Pixel).unwrap();
            let shader = device.create_shader(&binary, gpu::ShaderVariant::Pixel, "main".to_string()).unwrap();

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
            for Item(name, variant) in exports.iter() {
                let mut name_uppercase = name.clone();
                name_uppercase.make_ascii_uppercase();

                let line= format!(
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
            if attributes.len() > 0 || system_semantics.len() > 0{
                source.push_str("struct VertexInput {\n");
                for Item(name, variant) in attributes.iter() {
                    let mut name_uppercase = name.clone();
                    name_uppercase.make_ascii_uppercase();
    
                    let line= format!(
                        "    {} {} : {};\n", 
                        variant.into_type_string(), 
                        name, 
                        name_uppercase
                    );
                    source.push_str(&line);
                }

                for semantic in system_semantics.iter() {
                    let line = match semantic {
                        SystemSemantics::VertexId => "uint vertex_id : SV_VertexID;\n"
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
            let binary = gpu::shaders::compile("vertex.hlsl", &source, "main", gpu::ShaderVariant::Vertex).unwrap();
            let shader = device.create_shader(&binary, gpu::ShaderVariant::Vertex, "main".to_string()).unwrap();

            shader
        };
        let shaders = vec![pixel_shader, vertex_shader];

        let (draw_mode, line_width, cull_mode, color_mask) = {
            let mut cull_mode = gpu::CullMode::empty();
            for it in render_states.cull_mode.iter() {
                match it {
                    CullModeSerde::Front => cull_mode.insert(gpu::CullMode::FRONT),
                    CullModeSerde::Back => cull_mode.insert(gpu::CullMode::BACK),
                }
            }

            let mut color_mask = gpu::ColorMask::empty();
            for it in render_states.color_mask.iter() {
                match it {
                    ColorMaskSerde::Red => color_mask.insert(gpu::ColorMask::RED),
                    ColorMaskSerde::Green => color_mask.insert(gpu::ColorMask::GREEN),
                    ColorMaskSerde::Blue => color_mask.insert(gpu::ColorMask::BLUE),
                    ColorMaskSerde::Alpha => color_mask.insert(gpu::ColorMask::ALPHA),
                }
            }

            (render_states.draw_mode, render_states.line_width, cull_mode, color_mask)
        };

        let pipeline_desc  = gpu::GraphicsPipelineDescription{
            render_pass,
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

            push_constant_size: 4,
        };

        let pipeline = device.create_pipeline(gpu::PipelineDescription::Graphics(pipeline_desc)).unwrap();

        (id, Pipeline{ file: pipeline_file, gpu: pipeline })
    }
}