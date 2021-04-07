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

pub enum Shape {
    Rect(RectShape),
    Text(TextShape),
}

pub struct PainterVertexVariant(i32);

impl PainterVertexVariant {
    pub const SOLID: Self = Self(0);
    pub const IMAGE: Self = Self(1);
    pub const FONT:  Self = Self(2);
}

pub struct PainterVertex {
    position: Vector3,
    color:    Color,
    variant:  PainterVertexVariant,
    texture:  i32,
}

impl Vertex for PainterVertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![VertexAttribute::Vector3, VertexAttribute::Color, VertexAttribute::Int32, VertexAttribute::Int32]
    }
}

pub struct Painter {
    shapes: Vec<Shape>
}

impl Painter {
    pub fn new() -> Self {
        Self {
            shapes: Vec::with_capacity(1024)
        }
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
}