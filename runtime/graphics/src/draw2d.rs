use gpu::{
	Buffer,
	BufferUsage,
};
use math::vec2;
use math::{
	Color,
	Rect,
	Vector2,
	Vector4,
};
use resources::Handle;

use crate::FontCollection;

#[derive(Clone)]
pub struct PainterStyle {
	pub color: Color,

	pub line_width: f32,

	pub font_collection: Handle<FontCollection>,
	pub font_size: u32,
}

impl Default for PainterStyle {
	fn default() -> Self {
		Self {
			color: Color::WHITE,
			line_width: 1.0,

			font_collection: Handle::default(),
			font_size: 12,
		}
	}
}

pub struct PainterVertex {
	pub position: Vector2,
	pub uv: Vector2,
	pub scissor: Vector4,
	pub color: Color,
	pub texture: u32,
}

#[derive(Default)]
pub struct Painter {
	vertices: Vec<PainterVertex>,
	indices: Vec<u32>,

	scissor: Option<Rect>,
}

impl Painter {
	pub fn new() -> Self {
		Self {
			vertices: Vec::with_capacity(1024),
			indices: Vec::with_capacity(1024),

			scissor: None,
		}
	}

	pub fn scissor(
		&mut self,
		scissor: impl Into<Rect>,
		paint: impl FnOnce(&mut Painter),
	) -> &mut Self {
		let old = self.scissor;
		self.scissor = Some(scissor.into());
		paint(self);
		self.scissor = old;
		self
	}

	pub fn fill_rect(&mut self, style: &PainterStyle, rect: impl Into<Rect>) -> &mut Self {
		let rect = rect.into();
		let bottom_left = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_left(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: rect.top_left(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_right(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: rect.top_right(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;

		self.indices.push(bottom_left);
		self.indices.push(top_left);
		self.indices.push(top_right);

		self.indices.push(bottom_left);
		self.indices.push(top_right);
		self.indices.push(bottom_right);

		self
	}

	pub fn stroke(
		&mut self,
		style: &PainterStyle,
		a: impl Into<Vector2>,
		b: impl Into<Vector2>,
	) -> &mut Self {
		let a = a.into();
		let b = b.into();

		let width = style.line_width / 2.0;
		let perp = (b - a).norm().perp() * width;

		let bottom_left = {
			self.vertices.push(PainterVertex {
				position: a - perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: b - perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: a + perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: b + perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::INFINITY).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;

		self.indices.push(bottom_left);
		self.indices.push(top_left);
		self.indices.push(top_right);

		self.indices.push(bottom_left);
		self.indices.push(top_right);
		self.indices.push(bottom_right);

		self
	}

	pub fn stroke_rect(&mut self, style: &PainterStyle, rect: impl Into<Rect>) -> &mut Self {
		let half = style.line_width / 2.0;
		let x = vec2!(half, 0.0);
		let y = vec2!(0.0, half);
		let rect = rect.into();

		let bl = rect.bottom_left();
		let tl = rect.top_left();
		let br = rect.bottom_right();
		let tr = rect.top_right();

		self.stroke(style, bl + x, tl + x)
			.stroke(style, br - x, tr - x)
			.stroke(style, bl + y, br + y)
			.stroke(style, tl - y, tr - y)
	}

	pub fn finish(self) -> gpu::Result<(Buffer<PainterVertex>, Buffer<u32>)> {
		let vertex_buffer = Buffer::new(
			BufferUsage::VERTEX,
			gpu::MemoryType::HostVisible,
			self.vertices.len(),
		)?;
		vertex_buffer.copy_to(&self.vertices)?;

		let index_buffer = Buffer::new(
			BufferUsage::INDEX,
			gpu::MemoryType::HostVisible,
			self.indices.len(),
		)?;
		index_buffer.copy_to(&self.indices)?;
		Ok((vertex_buffer, index_buffer))
	}
}
