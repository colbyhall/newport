use gpu::{
	Buffer,
	BufferUsage,
};
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
	color: Color,

	line_width: f32,

	font_collection: Handle<FontCollection>,
	font_size: u32,
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
