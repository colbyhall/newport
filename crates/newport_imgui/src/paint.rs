use gpu::GraphicsContext;

use crate::math::{ Rect, Color, Vector2, Matrix4, Vector3 };
use crate::graphics::{ Texture, Graphics };
use crate::asset::AssetRef;
use crate::engine::Engine;

use crate::gpu;

use std::mem::size_of;

pub struct RectShape {
    bounds:     Rect,
    scissor:    Rect,

    _roundness:  Rect, // TODO: Rounded rects
    color:      Color,
    texture:    Option<AssetRef<Texture>>,
}

impl RectShape {
    pub fn scissor(&mut self, scissor: impl Into<Rect>) -> &mut Self {
        self.scissor = scissor.into();
        self
    }
    
    pub fn roundness(&mut self, corners: impl Into<Rect>) -> &mut Self {
        self._roundness = corners.into();
        self
    }

    pub fn color(&mut self, color: impl Into<Color>) -> &mut Self {
        self.color = color.into();
        self
    }

    pub fn texture(&mut self, texture: &AssetRef<Texture>) -> &mut Self {
        self.texture = Some(texture.clone());
        self
    }

    fn tesselate(&self, canvas: &mut Mesh) {
        let top_left_pos  = self.bounds.top_left();
        let top_right_pos = self.bounds.top_right();
        let bot_left_pos  = self.bounds.bottom_left();
        let bot_right_pos = self.bounds.bottom_right();

        let top_left_uv  = (0.0, 1.0).into();
        let top_right_uv = (1.0, 1.0).into();
        let bot_left_uv  = (0.0, 0.0).into();
        let bot_right_uv = (1.0, 0.0).into();

        let texture = {
            match &self.texture {
                Some(texture) => {
                    let texture = texture.read();
                    texture.gpu().bindless().unwrap()
                },
                None => 0,
            }
        };

        let indices_start = canvas.vertices.len() as u32;

        canvas.vertices.push(Vertex{
            position:   top_left_pos,
            uv:         top_left_uv,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.vertices.push(Vertex{
            position:   top_right_pos,
            uv:         top_right_uv,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.vertices.push(Vertex{
            position:   bot_left_pos,
            uv:         bot_left_uv,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.vertices.push(Vertex{
            position:   bot_right_pos,
            uv:         bot_right_uv,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });

        canvas.indices.push(indices_start + 2);
        canvas.indices.push(indices_start + 0);
        canvas.indices.push(indices_start + 1);

        canvas.indices.push(indices_start + 2);
        canvas.indices.push(indices_start + 1);
        canvas.indices.push(indices_start + 3);
    }
}

enum Shape {
    Rect(RectShape),
}

pub struct Painter {
    shapes: Vec<Shape>,
}

impl Painter {
    pub fn new() -> Self {
        Self {
            shapes: Vec::with_capacity(128)
        }
    }

    pub fn rect(&mut self, bounds: impl Into<Rect>) -> &mut RectShape {
        let bounds = bounds.into();

        let shape = RectShape {
            bounds:     bounds,
            scissor:    bounds,

            _roundness:  Rect::default(),
            color:       Color::WHITE,
            texture:     None,
        };
        self.shapes.push(Shape::Rect(shape));

        match self.shapes.last_mut().unwrap() {
            Shape::Rect(result) => result,
            _ => unreachable!()
        }
    }

    pub fn tesselate(mut self, canvas: &mut Mesh) {
        self.shapes.drain(..).for_each(|it| {
            match it {
                Shape::Rect(rect) => rect.tesselate(canvas),
            }
        })
    }
}

pub struct Vertex {
    pub position: Vector2,
    pub uv:       Vector2,
    pub scissor:  Rect,
    pub color:    Color,
    pub texture:  u32,
}

impl gpu::Vertex for Vertex {
    fn attributes() -> Vec<gpu::VertexAttribute> {
        vec![
            gpu::VertexAttribute::Vector2,
            gpu::VertexAttribute::Vector2,
            gpu::VertexAttribute::Vector4,
            gpu::VertexAttribute::Color,
            gpu::VertexAttribute::Uint32,
        ]
    }
}

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices:  Vec<u32>,
}

static SHADER_SOURCE: &str = "
    #define NULL 0
    ByteAddressBuffer all_buffers[]  : register(t0);
    Texture2D         all_textures[] : register(t1);
    SamplerState      all_samplers[] : register(s2);
    struct Constants {
        float4x4 view;
        float2   viewport;
    };
    [[vk::push_constant]] Constants constants;

    struct Vertex {
        float2 position : POSITION;
        float2 uv       : TEXCOORD;
        float4 scissor  : SCISSOR;
        float4 color    : COLOR;
        uint texture    : TEXTURE;
    };

    struct Vertex_Out {
        float2 uv       : TEXCOORD;
        float4 color    : COLOR;
        float4 scissor  : SCISSOR;
        float2 pos      : POS;
        uint texture    : TEXTURE;
        
        float4 position : SV_POSITION;
    };

    Vertex_Out main_vs( Vertex IN ){
        Vertex_Out OUT;
        OUT.uv      = IN.uv;
        OUT.texture = IN.texture;
        OUT.color   = IN.color;
        OUT.scissor = IN.scissor;
        OUT.pos     = IN.position.xy;

        OUT.position = mul(constants.view, float4(IN.position, 10.0, 1.0));

        return OUT;
    }

    struct Pixel_In {
        float2 uv      : TEXCOORD;
        float4 color   : COLOR;
        float4 scissor : SCISSOR;
        float2 pos     : POS;
        uint texture   : TEXTURE;
    };
    float4 main_ps( Pixel_In IN) : SV_TARGET {
        Texture2D    my_texture = all_textures[IN.texture];
        SamplerState my_sampler = all_samplers[IN.texture];

        if (IN.pos.x >= IN.scissor.x && IN.pos.y >= IN.scissor.y && IN.pos.x <= IN.scissor.z && IN.pos.y <= IN.scissor.w) {
            if (IN.texture == NULL) {
                return IN.color;
            } else {
                return IN.color * my_texture.Sample(my_sampler, IN.uv, 0);
            }
        }
        
        return float4(0.0, 0.0, 0.0, 0.0);
    }
";

pub struct DrawState {
    pipeline: gpu::Pipeline,
}

#[allow(dead_code)]
struct DrawConstants {
    view:       Matrix4,
    viewport:   Vector2,
}

impl DrawState {
    pub fn new() -> Self {
        let engine = Engine::as_ref();
        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let vertex_main = "main_vs".to_string();
        let pixel_main  = "main_ps".to_string();
        
        let vertex_bin = gpu::shaders::compile("vertex.hlsl", SHADER_SOURCE, &vertex_main, gpu::ShaderVariant::Vertex).unwrap();
        let pixel_bin  = gpu::shaders::compile("pixel.hlsl", SHADER_SOURCE, &pixel_main, gpu::ShaderVariant::Pixel).unwrap();

        let vertex_shader = device.create_shader(&vertex_bin[..], gpu::ShaderVariant::Vertex, vertex_main).unwrap();
        let pixel_shader  = device.create_shader(&pixel_bin[..], gpu::ShaderVariant::Pixel, pixel_main).unwrap();

        let pipeline_desc = gpu::PipelineBuilder::new_graphics(graphics.backbuffer_render_pass())
            .shaders(vec![vertex_shader, pixel_shader])
            .vertex::<Vertex>()
            .enable_blend()
            .dst_alpha_blend(gpu::BlendFactor::OneMinusSrcAlpha)
            .push_constant_size::<DrawConstants>()
            .build();

        let pipeline = device.create_pipeline(pipeline_desc).unwrap();
        Self { pipeline }
    }

    pub fn draw(&self, mesh: Mesh, gfx: &mut GraphicsContext) {
        let engine = Engine::as_ref();
        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let window = engine.window();
        let dpi    = window.dpi();

        let vertex_buffer = device.create_buffer(
            gpu::BufferUsage::VERTEX, 
            gpu::MemoryType::HostVisible, 
            mesh.vertices.len() * size_of::<Vertex>()
        ).unwrap();
        vertex_buffer.copy_to(&mesh.vertices[..]);

        let index_buffer = device.create_buffer(
            gpu::BufferUsage::INDEX, 
            gpu::MemoryType::HostVisible, 
            mesh.indices.len() * size_of::<u32>()
        ).unwrap();
        index_buffer.copy_to(&mesh.indices[..]);

        let viewport = window.size();
        let viewport = Vector2::new(viewport.0 as f32 / dpi, viewport.1 as f32 / dpi);

        let proj = Matrix4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
        let view = Matrix4::translate(Vector3::new(-viewport.x / 2.0, -viewport.y / 2.0, 0.0));

        gfx.bind_pipeline(&self.pipeline);
        gfx.bind_vertex_buffer(&vertex_buffer);
        gfx.bind_index_buffer(&index_buffer);
        gfx.push_constants(DrawConstants{
            view:     proj * view,
            viewport: viewport,
        });
        gfx.draw_indexed(mesh.indices.len(), 0);
    }
}
