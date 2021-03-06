use crate::{
    gpu,
    math,
    graphics,
    asset::{ AssetRef, AssetManager },
    engine::Engine,

    Context,
};

use gpu::{
    GraphicsContext,
    Texture,
};

use graphics::{
    FontCollection,
    Graphics,
    Pipeline,
};

use math::{
    Rect, 
    Color, 
    Vector2, 
    Matrix4, 
    Vector3
};


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
    bounds:  Rect,
    scissor: Rect,

    roundness:  Roundness,
    color:      Color,
    texture:    Option<Texture>,
}

impl RectShape {  
    fn tesselate(&self, canvas: &mut Canvas) {
        let texture = {
            match &self.texture {
                Some(texture) => texture.bindless().unwrap_or(0),
                None => 0,
            }
        };

        let max = self.roundness.max();
        if max <= 0.0 {
            canvas.rect(self.bounds, (0.0, 0.0, 1.0, 1.0).into(), self.scissor, self.color, texture);
            return;
        }

        let size = self.bounds.size();
        let radius = max.min(size.x.min(size.y) / 2.0);

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

        let top_right_radius = self.roundness.top_right.min(radius);
        let top_right = self.bounds.top_right() - top_right_radius;
        let (top_right_first, top_right_second) = corner(0.0, math::PI / 2.0, top_right, top_right_radius);

        let top_left_radius = self.roundness.top_left.min(radius);
        let top_left = self.bounds.top_left() + Vector2::new(top_left_radius, -top_left_radius);
        let (top_left_first, top_left_second) = corner(math::PI * 1.5, math::TAU, top_left, top_left_radius);

        let bottom_left_radius = self.roundness.bottom_left.min(radius);
        let bottom_left = self.bounds.bottom_left() + bottom_left_radius;
        let (bottom_left_first, bottom_left_second) = corner(math::PI, math::PI * 1.5, bottom_left, bottom_left_radius);
        
        let bottom_right_radius = self.roundness.bottom_right.min(radius);
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
    at: Vector2,

    scissor: Rect,

    font: AssetRef<FontCollection>,
    size: u32,
    dpi:  f32,

    color:      Color,
}

impl TextShape {
    pub fn tesselate(&self, canvas: &mut Canvas) {
        let mut font_collection = self.font.write();
        let font = font_collection.font_at_size(self.size, self.dpi).unwrap();

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

pub struct TriangleShape {
    points: [Vector2; 3],
    scissor: Rect,

    color: Color,
}

impl TriangleShape {
    pub fn tesselate(&self, canvas: &mut Canvas) {
        for point in self.points.iter() {
            canvas.vertices.push(Vertex{
                position:   *point,
                uv:         Vector2::ZERO,
                color:      self.color,
                scissor:    self.scissor,
                texture:    0,
            });
            canvas.indices.push((canvas.vertices.len() - 1) as u32);
        }
    }
}

pub enum Shape {
    Rect(RectShape),
    Triangle(TriangleShape),
    Text(TextShape),
}

impl Shape {
    pub fn solid_rect(bounds: impl Into<Rect>, color: impl Into<Color>, roundness: impl Into<Roundness>) -> Self {
        Self::Rect(RectShape{
            bounds:    bounds.into(),
            scissor:   Rect::INFINITY,

            roundness: roundness.into(),
            color:     color.into(),
            texture:   None,
        })
    }

    pub fn solid_triangle(points: [Vector2; 3], color: impl Into<Color>) -> Self {
        Self::Triangle(TriangleShape{
            points:  points,
            scissor: Rect::INFINITY,

            color:   color.into(),
        })
    }

    pub fn textured_rect(bounds: impl Into<Rect>, color: impl Into<Color>, roundness: impl Into<Roundness>, texture: &Texture) -> Self {
        Self::Rect(RectShape{
            bounds:    bounds.into(),
            scissor:   Rect::INFINITY,

            roundness: roundness.into(),
            color:     color.into(),
            texture:   Some(texture.clone()),
        })
    }

    pub fn text(text: impl Into<String>, at: impl Into<Vector2>, font: &AssetRef<FontCollection>, size: u32, dpi: f32, color: impl Into<Color>) -> Self {
        Self::Text(TextShape{
            text: text.into(),
            at:   at.into(),

            scissor:   Rect::INFINITY,

            font: font.clone(),
            size: size,
            dpi: dpi,
            color: color.into(),
        })
    }

    fn set_scissor(&mut self, scissor: Rect) {
        match self {
            Shape::Text(shape) => shape.scissor = scissor,
            Shape::Rect(shape) => shape.scissor = scissor,
            Shape::Triangle(shape) => shape.scissor = scissor,
        }
    }
}

pub struct Painter {
    shapes:  Vec<Shape>,
    scissors: Vec<Rect>,
}

impl Painter {
    pub fn new() -> Self {
        let mut scissors = Vec::new();
        scissors.push(Rect::INFINITY);
        Self {
            shapes:   Vec::with_capacity(128),
            scissors: scissors,
        }
    }

    pub fn num_shapes(&self) -> usize {
        self.shapes.len()
    }

    pub fn push_shape(&mut self, mut shape: Shape) {
        let scissor = *self.scissors.last().unwrap();
        shape.set_scissor(scissor);
        self.shapes.push(shape);
    }

    pub fn insert_shape(&mut self, index: usize, mut shape: Shape) {
        let scissor = *self.scissors.last().unwrap();
        shape.set_scissor(scissor);
        self.shapes.insert(index, shape);
    }

    pub fn tesselate(mut self, canvas: &mut Canvas) {
        self.shapes.drain(..).for_each(|it| {
            match it {
                Shape::Rect(rect) => rect.tesselate(canvas),
                Shape::Text(text) => text.tesselate(canvas),
                Shape::Triangle(triangle) => triangle.tesselate(canvas),
            }
        })
    }

    pub fn push_scissor(&mut self, scissor: Rect) {
        self.scissors.push(scissor);
    }

    pub fn pop_scissor(&mut self) {
        self.scissors.pop();
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
pub struct Canvas {
    pub vertices: Vec<Vertex>,
    pub indices:  Vec<u32>,

    pub width: u32,
    pub height: u32,
}

impl Canvas {
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

pub struct DrawState {
    pipeline:    AssetRef<Pipeline>,
    render_pass: gpu::RenderPass,
}

impl DrawState {
    pub fn new() -> Self {
        let engine = Engine::as_ref();
        let graphics: &Graphics = engine.module().unwrap();
        let device = graphics.device();
        let asset_manager: &AssetManager = engine.module().unwrap();

        Self { pipeline: asset_manager.find("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}").unwrap(), render_pass: device.create_render_pass(vec![gpu::Format::RGBA_U8], None).unwrap() }
    }

    pub fn record(&self, canvas: Canvas, gfx: &mut GraphicsContext, ctx: &Context) -> Result<gpu::Texture, ()> {
        let graphics = Engine::as_ref().module::<Graphics>().unwrap();
        let device = graphics.device();

        if canvas.vertices.len() == 0 {
            return Err(());
        }

        let vertex_buffer = device.create_buffer(
            gpu::BufferUsage::VERTEX, 
            gpu::MemoryType::HostVisible, 
            canvas.vertices.len() * size_of::<Vertex>()
        ).unwrap();
        vertex_buffer.copy_to(&canvas.vertices[..]);

        let index_buffer = device.create_buffer(
            gpu::BufferUsage::INDEX, 
            gpu::MemoryType::HostVisible, 
            canvas.indices.len() * size_of::<u32>()
        ).unwrap();
        index_buffer.copy_to(&canvas.indices[..]);

        let viewport = ctx.input.viewport.size();

        let proj = Matrix4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
        let view = Matrix4::translate(Vector3::new(-viewport.x / 2.0, -viewport.y / 2.0, 0.0));

        struct Import {
            _view: Matrix4,
        }
        let import_buffer = device.create_buffer(
            gpu::BufferUsage::CONSTANTS, 
            gpu::MemoryType::HostVisible, 
            size_of::<Import>()
        ).unwrap();
        import_buffer.copy_to(&[Import{
            _view: proj * view,
        }]);

        let pipeline = self.pipeline.read();

        let backbuffer = device.create_texture(
            gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_ATTACHMENT, 
            gpu::MemoryType::DeviceLocal, 
            gpu::Format::RGBA_U8, 
            canvas.width, 
            canvas.height, 
            1, 
            gpu::Wrap::Clamp,
            gpu::Filter::Linear,
            gpu::Filter::Linear
        ).unwrap();

        gfx.begin_render_pass(&self.render_pass, &[&backbuffer]);
            gfx.clear(Color::BLACK);
            gfx.bind_pipeline(&pipeline.gpu);
            gfx.bind_vertex_buffer(&vertex_buffer);
            gfx.bind_index_buffer(&index_buffer);
            gfx.bind_constant_buffer(&import_buffer);
            gfx.draw_indexed(canvas.indices.len(), 0);
        gfx.end_render_pass();
        gfx.resource_barrier_texture(&backbuffer, gpu::Layout::ColorAttachment, gpu::Layout::ShaderReadOnly);
        
        Ok(backbuffer)
    }
}
