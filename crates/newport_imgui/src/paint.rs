use newport_math::{ Color, Rect, Vector2, Vector3 };
use newport_graphics::font::FontCollection;
use newport_asset::AssetRef;
use newport_gpu::*;

pub struct RectShape {
    bounds:  Rect,

    color: Option<Color>,
}

impl RectShape {
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }
}

pub struct TextShape {
    text: String,

    font: AssetRef<FontCollection>,
    size: u32,

    at:   Vector2,

    color: Option<Color>,
}

impl TextShape {
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }
}

pub enum Shape {
    Rect(RectShape),
    Text(TextShape),
}

#[derive(Copy, Clone, Default, Debug)]
pub struct PainterVertexVariant(i32);

impl PainterVertexVariant {
    pub const SOLID: Self = Self(0);
    pub const IMAGE: Self = Self(1);
    pub const FONT:  Self = Self(2);
}

pub struct PainterVertex {
    pub position: Vector3,
    pub uv:       Vector2,
    pub color:    Color,
    pub variant:  PainterVertexVariant,
    pub texture:  i32,
}

impl Vertex for PainterVertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::Vector3, 
            VertexAttribute::Vector2, 
            VertexAttribute::Color, 
            VertexAttribute::Int32, 
            VertexAttribute::Int32
        ]
    }
}

pub struct Painter {
    shapes: Vec<Shape>,
}

impl Painter {
    pub fn new() -> Self {
        Self { shapes: Vec::with_capacity(1024) }
    }

    pub fn rect(&mut self, bounds: Rect) -> &mut RectShape {
        self.shapes.push(
            Shape::Rect(RectShape{
                bounds: bounds,

                color: None,
            }
        ));

        match self.shapes.last_mut().unwrap() {
            Shape::Rect(ret) => ret,
            _ => unreachable!()
        }
    }

    pub fn text(&mut self, text: String, font: AssetRef<FontCollection>, size: u32, at: Vector2) -> &mut TextShape {
        self.shapes.push(
            Shape::Text(TextShape{
                text: text,

                font: font,
                size: size,

                at: at,

                color: None,
            }
        ));

        match self.shapes.last_mut().unwrap() {
            Shape::Text(ret) => ret,
            _ => unreachable!()
        }
    }

    pub fn tesselate(self) -> Vec<PainterVertex> {
        // TODO: Maybe cache this?
        let mut cap = 0;
        for it in self.shapes.iter() {
            match it {
                Shape::Rect(_) =>    cap += 6,
                Shape::Text(text) => cap += text.text.chars().count() * 6,
            }
        }

        fn push_rect(buffer: &mut Vec<PainterVertex>, rect: Rect, z: f32, uv: Rect, color: Color, variant: PainterVertexVariant, texture: i32) {
            let tl_pos = Vector3::new(rect.min.x, rect.max.y, z);
            let tr_pos = (rect.max, z).into();
            let bl_pos = (rect.min, z).into();
            let br_pos = Vector3::new(rect.max.x, rect.min.y, z);

            let tl_uv = Vector2::new(uv.min.x, uv.max.y);
            let tr_uv = uv.max;
            let bl_uv = uv.min;
            let br_uv = Vector2::new(uv.max.x, uv.min.y);

            // First triangle
            buffer.push(PainterVertex{
                position: tl_pos,
                uv:       tl_uv,
                color:    color,
                variant:  variant,
                texture:  texture,
            });

            buffer.push(PainterVertex{
                position: bl_pos,
                uv:       bl_uv,
                color:    color,
                variant:  variant,
                texture:  texture,
            });

            buffer.push(PainterVertex{
                position: tr_pos,
                uv:       tr_uv,
                color:    color,
                variant:  variant,
                texture:  texture,
            });

            // Second triangle
            buffer.push(PainterVertex{
                position: bl_pos,
                uv:       bl_uv,
                color:    color,
                variant:  variant,
                texture:  texture,
            });

            buffer.push(PainterVertex{
                position: br_pos,
                uv:       br_uv,
                color:    color,
                variant:  variant,
                texture:  texture,
            });

            buffer.push(PainterVertex{
                position: tr_pos,
                uv:       tr_uv,
                color:    color,
                variant:  variant,
                texture:  texture,
            });
        }

        let z = 10.0;

        let mut buffer = Vec::with_capacity(cap);
        for it in self.shapes.iter() {
            match it {
                Shape::Rect(rect) => {
                    let color = rect.color.unwrap_or(Color::WHITE);
                    push_rect(&mut buffer, rect.bounds, z, Rect::default(), color, PainterVertexVariant::SOLID, 0);
                },
                Shape::Text(text) => {
                    let mut font_collection = text.font.write();
                    let font = font_collection.font_at_size(text.size, 1.0).unwrap();

                    let color = text.color.unwrap_or(Color::WHITE);

                    let mut pos = text.at;
                    for c in text.text.chars() {
                        match c {
                            '\n' => {
                                pos.x = text.at.x;
                                pos.y -= text.size as f32;
                            },
                            '\r' => pos.x = text.at.x,
                            '\t' => {
                                let g = font.glyph_from_char(' ').unwrap();
                                pos.x += g.advance;
                            },
                            _ => {
                                let g = font.glyph_from_char(c).unwrap();

                                let xy = Vector2::new(pos.x, pos.y - text.size as f32);
                                
                                let x0 = xy.x + g.bearing_x;
                                let y1 = xy.y + g.bearing_y;
                                let x1 = x0 + g.width;
                                let y0 = y1 - g.height;
                                let rect = (Vector2::new(x0, y0), Vector2::new(x1, y1)).into();

                                push_rect(&mut buffer, rect, z, g.uv, color, PainterVertexVariant::FONT, font.atlas.bindless().unwrap() as i32);
                                pos.x += g.advance;
                            }
                        }
                    }
                },
            }
        }

        buffer
    }
}