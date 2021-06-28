use std::mem::size_of;
use crate::*;


use newport_serde::{
    self as serde,
    Serialize,
    Deserialize
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum DrawMode {
    Fill,
    Line,
    Point,
}

bitflags!{
    pub struct CullMode: u32 {
        const FRONT = 0b01;
        const BACK  = 0b10;
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub enum CompareOp {
    Never,
    Less,             // A < B
    Equal,            // A == B
    LessOrEqual,      // A < B || A == B
    Greater,          // A > B
    NotEqual,         // A != B
    GreaterOrEqual,   // A > B || A == B
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

#[derive(Copy, Clone, Debug)]
pub enum VertexAttribute {
    Int32,
    Uint32,
    Float32,
    Vector2,
    Vector3,
    Vector4,
    Color,
}

impl VertexAttribute {
    pub fn size(self) -> usize {
        match self {
            VertexAttribute::Int32   => 4,
            VertexAttribute::Uint32  => 4,
            VertexAttribute::Float32 => 4,
            VertexAttribute::Color   => 16,
            VertexAttribute::Vector2 => 8,
            VertexAttribute::Vector3 => 12,
            VertexAttribute::Vector4 => 16,
        }
    }
}

pub trait Vertex {
    fn attributes() -> Vec<VertexAttribute>;
}

pub struct PipelineBuilder {
    desc: PipelineDescription,
}

impl PipelineBuilder {
    pub fn new_graphics(render_pass: &RenderPass) -> Self {
        let desc = GraphicsPipelineDescription{
            render_pass: render_pass.clone(),
            shaders:     Vec::new(),

            vertex_attributes: Vec::new(),

            draw_mode:  DrawMode::Fill,
            line_width: 1.0,

            cull_mode:  CullMode::empty(),
            color_mask: ColorMask::all(),

            blend_enabled: false,

            src_color_blend_factor: BlendFactor::One,
            dst_color_blend_factor: BlendFactor::One,
            color_blend_op:         BlendOp::Add,

            src_alpha_blend_factor: BlendFactor::One,
            dst_alpha_blend_factor: BlendFactor::One,
            alpha_blend_op:         BlendOp::Add,

            depth_test:    false, 
            depth_write:   false,
            depth_compare: CompareOp::Less,

            push_constant_size: 0,
        };
        Self { desc: PipelineDescription::Graphics(desc) }
    }

    pub fn shaders(mut self, shaders: Vec<Shader>) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.shaders = shaders,
            _ => unreachable!()
        }
        self
    }

    pub fn vertex<T: Vertex>(mut self) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.vertex_attributes = T::attributes(),
            _ => unreachable!()
        }
        self
    }

    pub fn draw_mode(mut self, mode: DrawMode) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.draw_mode = mode,
            _ => unreachable!()
        }
        self
    }

    pub fn line_width(mut self, width: f32) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.line_width = width,
            _ => unreachable!()
        }
        self
    }
    
    pub fn push_constant_size<T: Sized>(mut self) -> Self {
        assert!(size_of::<T>() <= 128);
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.push_constant_size = size_of::<T>(),
            _ => unreachable!()
        }
        self
    }

    pub fn enable_blend(mut self) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.blend_enabled = true,
            _ => unreachable!()
        }
        self
    }

    pub fn src_alpha_blend(mut self, factor: BlendFactor) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.src_alpha_blend_factor = factor,
            _ => unreachable!()
        }
        self
    }

    pub fn dst_alpha_blend(mut self, factor: BlendFactor) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.dst_alpha_blend_factor = factor,
            _ => unreachable!()
        }
        self
    }

    pub fn src_color_blend(mut self, factor: BlendFactor) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.src_color_blend_factor = factor,
            _ => unreachable!()
        }
        self
    }

    pub fn dst_color_blend(mut self, factor: BlendFactor) -> Self {
        match &mut self.desc {
            PipelineDescription::Graphics(gfx) => gfx.dst_color_blend_factor = factor,
            _ => unreachable!()
        }
        self
    }

    pub fn build(self) -> PipelineDescription {
        self.desc
    }
}

#[derive(Debug)]
pub struct GraphicsPipelineDescription {
    pub render_pass:  RenderPass,
    pub shaders:      Vec<Shader>,
    
    pub vertex_attributes: Vec<VertexAttribute>,

    pub draw_mode:  DrawMode,
    pub line_width: f32,

    pub cull_mode:   CullMode,
    pub color_mask:  ColorMask,

    pub blend_enabled: bool,

    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op:         BlendOp,

    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op:         BlendOp,    

    pub depth_test:    bool,
    pub depth_write:   bool,
    pub depth_compare: CompareOp,

    /// Capped at 128 bytes
    pub push_constant_size : usize, 
}

#[derive(Debug)]
pub enum PipelineDescription {
    Graphics(GraphicsPipelineDescription),
    Compute,
}

#[derive(Clone)]
pub struct Pipeline(pub(crate) Arc<api::Pipeline>);