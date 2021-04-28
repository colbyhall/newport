use gpu::GraphicsContext;

use crate::math::{ Rect, Color, Vector2, Matrix4, Vector3 };
use crate::graphics::{ Texture, Graphics, FontCollection };
use crate::asset::AssetRef;
use crate::engine::Engine;
use crate::math;

use crate::{ gpu, Context };

use std::mem::size_of;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Roundness {
    pub bottom_left:  f32,
    pub bottom_right: f32, 
    pub top_left:  f32,
    pub top_right: f32, 
}

impl Roundness {
    pub fn max(self) -> f32 {
        let mut max = self.bottom_left;
        if max < self.bottom_right {
            max = self.bottom_right;
        }
        if max < self.top_left {
            max = self.top_left;
        }
        if max < self.top_right {
            max = self.top_right;
        }
        max
    }
}

impl From<f32> for Roundness {
    fn from(rad: f32) -> Self {
        Self{
            bottom_left:  rad,
            bottom_right: rad,
            top_left:  rad,
            top_right: rad,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Roundness {
    fn from(xyzw: (f32, f32, f32, f32)) -> Self {
        Self{
            bottom_left:  xyzw.0,
            bottom_right: xyzw.1,
            top_left:  xyzw.2,
            top_right: xyzw.3,
        }
    }
}

pub struct RectShape {
    bounds:     Rect,
    scissor:    Rect,

    roundness:  Roundness,
    color:      Color,
    texture:    Option<AssetRef<Texture>>,
}

impl RectShape {
    pub fn scissor(&mut self, scissor: impl Into<Rect>) -> &mut Self {
        self.scissor = scissor.into();
        self
    }
    
    pub fn roundness(&mut self, corners: impl Into<Roundness>) -> &mut Self {
        self.roundness = corners.into();
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
        let texture = {
            match &self.texture {
                Some(texture) => {
                    let texture = texture.read();
                    texture.gpu().bindless().unwrap()
                },
                None => 0,
            }
        };

        let max = self.roundness.max();
        if max <= 0.0 {
            canvas.rect(self.bounds, (0.0, 0.0, 1.0, 1.0).into(), self.scissor, self.color, texture);
            return;
        }

        let size = self.bounds.size();
        let radius = math::min(max, math::min(size.x, size.y) / 2.0);

        canvas.vertices.push(Vertex{
            position:   self.bounds.pos(),
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        let center_index = canvas.vertices.len() as u32 - 1;

        let mut corner = |low: f32, high: f32, at: Vector2, r: f32|{
            let denom = math::PI / 50.0;
            let count = ((high - low) / denom) as usize;
            
            canvas.vertices.push(Vertex{
                position:   at + Vector2::new(low.sin(), low.cos()) * r,
                uv:         Vector2::ZERO,
                color:      self.color,
                scissor:    self.scissor,
                texture:    texture,
            });

            let first = canvas.vertices.len() as u32 - 1;

            for i in 0..count {
                canvas.indices.push(center_index);
                canvas.indices.push(canvas.vertices.len() as u32 - 1);

                let theta = (i + 1) as f32 * denom + low;
                canvas.vertices.push(Vertex{
                    position:   at + Vector2::new(theta.sin(), theta.cos()) * r,
                    uv:         Vector2::ZERO,
                    color:      self.color,
                    scissor:    self.scissor,
                    texture:    texture,
                }); 
                canvas.indices.push(canvas.vertices.len() as u32 - 1);
            }

            let first = canvas.vertices[first as usize].position;
            let second = canvas.vertices.last().unwrap().position;
            (first, second)
        };

        let top_right_radius = math::min(self.roundness.top_right, radius);
        let top_right = self.bounds.top_right() - top_right_radius;
        let (top_right_first, top_right_second) = corner(0.0, math::PI / 2.0, top_right, top_right_radius);

        let top_left_radius = math::min(self.roundness.top_left, radius);
        let top_left = self.bounds.top_left() + Vector2::new(top_left_radius, -top_left_radius);
        let (top_left_first, top_left_second) = corner(math::PI * 1.5, math::TAU, top_left, top_left_radius);

        let bottom_left_radius = math::min(self.roundness.bottom_left, radius);
        let bottom_left = self.bounds.bottom_left() + bottom_left_radius;
        let (bottom_left_first, bottom_left_second) = corner(math::PI, math::PI * 1.5, bottom_left, bottom_left_radius);
        
        let bottom_right_radius = math::min(self.roundness.bottom_right, radius);
        let bottom_right = self.bounds.bottom_right() + Vector2::new(-bottom_right_radius, bottom_right_radius);
        let (bottom_right_first, bottom_right_second) = corner(math::PI / 2.0, math::PI, bottom_right, bottom_right_radius);

        // Top triangle
        let at = canvas.vertices.len() as u32;
        canvas.vertices.push(Vertex{
            position:   top_left_second,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        }); 
        canvas.vertices.push(Vertex{
            position:   top_right_first,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.indices.push(center_index);
        canvas.indices.push(at + 0);
        canvas.indices.push(at + 1);

        // Right triangle
        let at = canvas.vertices.len() as u32;
        canvas.vertices.push(Vertex{
            position:   top_right_second,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        }); 
        canvas.vertices.push(Vertex{
            position:   bottom_right_first,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.indices.push(center_index);
        canvas.indices.push(at + 0);
        canvas.indices.push(at + 1);

        // Bottom triangle
        let at = canvas.vertices.len() as u32;
        canvas.vertices.push(Vertex{
            position:   bottom_right_second,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        }); 
        canvas.vertices.push(Vertex{
            position:   bottom_left_first,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.indices.push(center_index);
        canvas.indices.push(at + 0);
        canvas.indices.push(at + 1);

        // Left triangle
        let at = canvas.vertices.len() as u32;
        canvas.vertices.push(Vertex{
            position:   bottom_left_second,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        }); 
        canvas.vertices.push(Vertex{
            position:   top_left_first,
            uv:         Vector2::ZERO,
            color:      self.color,
            scissor:    self.scissor,
            texture:    texture,
        });
        canvas.indices.push(center_index);
        canvas.indices.push(at + 0);
        canvas.indices.push(at + 1);
    }
}

pub struct TextShape {
    text: String,

    at:         Vector2,
    scissor:    Rect,

    font: AssetRef<FontCollection>,
    size: u32,
    dpi:  f32,

    color:      Color,
}

impl TextShape {
    pub fn color(&mut self, color: impl Into<Color>) -> &mut Self {
        self.color = color.into();
        self
    }

    pub fn scissor(&mut self, scissor: impl Into<Rect>) -> &mut Self {
        self.scissor = scissor.into();
        self
    }

    pub fn tesselate(&self, canvas: &mut Mesh) {
        let mut font_collection = self.font.write();
        let font = font_collection.font_at_size(self.size, self.dpi).unwrap(); // TODO: DPI

        let mut pos = self.at;
        for c in self.text.chars() {
            match c {
                '\n' => {
                    pos.x = self.at.x;
                    pos.y -= self.size as f32;
                },
                '\r' => pos.x = self.at.x,
                '\t' => {
                    let g = font.glyph_from_char(' ').unwrap();
                    pos.x += g.advance;
                },
                _ => {
                    let g = font.glyph_from_char(c).unwrap();

                    let xy = Vector2::new(pos.x, pos.y - (font.height + font.descent));
                    
                    let x0 = xy.x + g.bearing_x;
                    let y1 = xy.y + g.bearing_y;
                    let x1 = x0 + g.width;
                    let y0 = y1 - g.height;
                    let bounds = (x0, y0, x1, y1).into();

                    canvas.rect(bounds, g.uv, self.scissor, self.color, font.atlas.bindless().unwrap_or_default());
                    pos.x += g.advance;
                }
            }
        }
    }
}

enum Shape {
    Rect(RectShape),
    Text(TextShape),
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

            roundness:   Roundness::default(),
            color:       Color::WHITE,
            texture:     None,
        };
        self.shapes.push(Shape::Rect(shape));

        match self.shapes.last_mut().unwrap() {
            Shape::Rect(result) => result,
            _ => unimplemented!()
        }
    }

    pub fn text(&mut self, text: String, at: Vector2, font: &AssetRef<FontCollection>, size: u32, dpi: f32) -> &mut TextShape {
        let shape = TextShape{
            text: text,

            at: at,
            scissor: (-f32::INFINITY, -f32::INFINITY, f32::INFINITY, f32::INFINITY).into(),

            font: font.clone(),
            size: size,
            dpi:  dpi,

            color: Color::WHITE,
        };
        self.shapes.push(Shape::Text(shape));
        
        match self.shapes.last_mut().unwrap() {
            Shape::Text(result) => result,
            _ => unimplemented!()
        }
    }

    pub fn tesselate(mut self, canvas: &mut Mesh) {
        self.shapes.drain(..).for_each(|it| {
            match it {
                Shape::Rect(rect) => rect.tesselate(canvas),
                Shape::Text(text) => text.tesselate(canvas),
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

impl Mesh {
    fn rect(self: &mut Self, bounds: Rect, uv: Rect, scissor: Rect, color: Color, texture: u32) {
        let size = bounds.size();
        if size.x <= 0.0 || size.y <= 0.0 {
            return;
        }

        let top_left_pos  = bounds.top_left();
        let top_right_pos = bounds.top_right();
        let bot_left_pos  = bounds.bottom_left();
        let bot_right_pos = bounds.bottom_right();

        let top_left_uv  = uv.top_left();
        let top_right_uv = uv.top_right();
        let bot_left_uv  = uv.bottom_left();
        let bot_right_uv = uv.bottom_right();

        let indices_start = self.vertices.len() as u32;

        self.vertices.push(Vertex{
            position:   top_left_pos,
            uv:         top_left_uv,
            color:      color,
            scissor:    scissor,
            texture:    texture,
        });
        self.vertices.push(Vertex{
            position:   top_right_pos,
            uv:         top_right_uv,
            color:      color,
            scissor:    scissor,
            texture:    texture,
        });
        self.vertices.push(Vertex{
            position:   bot_left_pos,
            uv:         bot_left_uv,
            color:      color,
            scissor:    scissor,
            texture:    texture,
        });
        self.vertices.push(Vertex{
            position:   bot_right_pos,
            uv:         bot_right_uv,
            color:      color,
            scissor:    scissor,
            texture:    texture,
        });

        self.indices.push(indices_start + 2);
        self.indices.push(indices_start + 0);
        self.indices.push(indices_start + 1);

        self.indices.push(indices_start + 2);
        self.indices.push(indices_start + 1);
        self.indices.push(indices_start + 3);
    }
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
            .dst_color_blend(gpu::BlendFactor::OneMinusSrcAlpha)
            .push_constant_size::<DrawConstants>()
            .build();

        let pipeline = device.create_pipeline(pipeline_desc).unwrap();
        Self { pipeline }
    }

    pub fn draw(&self, mesh: Mesh, gfx: &mut GraphicsContext, ctx: &Context) {
        let graphics = Engine::as_ref().module::<Graphics>().unwrap();
        let device = graphics.device();

        if mesh.vertices.len() == 0 {
            return;
        }

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

        let viewport = ctx.input.viewport.size();

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
