use gpu::{Buffer, BufferUsage, Texture};
use math::vec2;
use math::{Color, Rect, Vector2, Vector4};
use resources::Handle;

use crate::FontCollection;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Roundness {
	pub bottom_left: f32,
	pub bottom_right: f32,
	pub top_left: f32,
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
		Self {
			bottom_left: rad,
			bottom_right: rad,
			top_left: rad,
			top_right: rad,
		}
	}
}

impl From<(f32, f32, f32, f32)> for Roundness {
	fn from(xyzw: (f32, f32, f32, f32)) -> Self {
		Self {
			bottom_left: xyzw.0,
			bottom_right: xyzw.1,
			top_left: xyzw.2,
			top_right: xyzw.3,
		}
	}
}

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
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: rect.top_left(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_right(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: rect.top_right(),
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
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

	pub fn textured_rect(
		&mut self,
		style: &PainterStyle,
		rect: impl Into<Rect>,
		texture: &Handle<Texture>,
		uv: impl Into<Rect>,
	) -> &mut Self {
		let rect = rect.into();
		let uv = uv.into();

		let texture = texture.read().bindless().unwrap_or_default();

		let bottom_left = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_left(),
				uv: uv.bottom_left(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: rect.top_left(),
				uv: uv.top_left(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_right(),
				uv: uv.bottom_right(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: rect.top_right(),
				uv: uv.top_right(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture,
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
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: b - perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: a + perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color: style.color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: b + perp,
				uv: Vector2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
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
