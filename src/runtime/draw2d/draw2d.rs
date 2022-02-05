use {
	engine::{
		Builder,
		Module,
	},
	gpu::{
		Buffer,
		BufferUsage,
		Gpu,
		Texture,
	},
	math::{
		vec2,
		Color,
		Rect,
		Vec2,
		Vec4,
	},
	resources::{
		Importer,
		Resource,
	},
};

pub use font::*;

mod font;

// TODO: Bring font into this file
pub use font::*;

pub struct Draw2d;

impl Module for Draw2d {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Gpu>()
			.register(FontCollection::variant())
			.register(FontImporter::variant(&["ttf"]))
		// .register(Mesh::variant())
		// .register(MeshGltfImporter::variant(&["gltf", "glb"]))
	}
}

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

pub struct PainterVertex {
	pub position: Vec2,
	pub uv: Vec2,
	pub scissor: Vec4,
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

	pub fn fill_rect(&mut self, rect: impl Into<Rect>, color: impl Into<Color>) -> &mut Self {
		let color = color.into();

		let rect = rect.into();
		let bottom_left = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_left(),
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: rect.top_left(),
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_right(),
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: rect.top_right(),
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
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
		rect: impl Into<Rect>,
		texture: &Texture,
		uv: impl Into<Rect>,
		color: impl Into<Color>,
	) -> &mut Self {
		let rect = rect.into();
		let uv = uv.into();

		let texture = texture.bindless().unwrap_or_default();
		let color = color.into();

		let bottom_left = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_left(),
				uv: uv.bottom_left(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: rect.top_left(),
				uv: uv.top_left(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: rect.bottom_right(),
				uv: uv.bottom_right(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: rect.top_right(),
				uv: uv.top_right(),
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
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
		a: impl Into<Vec2>,
		b: impl Into<Vec2>,
		line_width: f32,
		color: impl Into<Color>,
	) -> &mut Self {
		let a = a.into();
		let b = b.into();

		let width = line_width / 2.0;
		let perp = (b - a).norm().perp() * width;

		let color = color.into();

		let bottom_left = {
			self.vertices.push(PainterVertex {
				position: a - perp,
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_left = {
			self.vertices.push(PainterVertex {
				position: b - perp,
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let bottom_right = {
			self.vertices.push(PainterVertex {
				position: a + perp,
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
				texture: 0,
			});
			self.vertices.len() - 1
		} as u32;
		let top_right = {
			self.vertices.push(PainterVertex {
				position: b + perp,
				uv: Vec2::ZERO,
				scissor: self.scissor.unwrap_or(Rect::MINMAX).into(),
				color,
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

	pub fn stroke_rect(
		&mut self,
		rect: impl Into<Rect>,
		line_width: f32,
		color: impl Into<Color>,
	) -> &mut Self {
		let half = line_width / 2.0;
		let x = vec2!(half, 0.0);
		let y = vec2!(0.0, half);
		let rect = rect.into();

		let bl = rect.bottom_left();
		let tl = rect.top_left();
		let br = rect.bottom_right();
		let tr = rect.top_right();

		let color = color.into();

		self.stroke(bl + x, tl + x, line_width, color)
			.stroke(br - x, tr - x, line_width, color)
			.stroke(bl + y, br + y, line_width, color)
			.stroke(tl - y, tr - y, line_width, color)
	}

	pub fn text(
		&mut self,
		text: &str,
		color: impl Into<Color>,
		at: impl Into<Vec2>,
		font: &Font,
	) -> &mut Self {
		let at = at.into();
		let color = color.into();

		let mut pos = at;
		for c in text.chars() {
			match c {
				'\n' => {
					pos.x = at.x;
					pos.y -= font.size as f32;
				}
				'\r' => pos.x = at.x,
				'\t' => {
					let g = font.glyph_from_char(' ').unwrap();
					pos.x += g.advance;
				}
				_ => {
					let g = font.glyph_from_char(c).unwrap();

					let xy = Vec2::new(pos.x, pos.y - (font.height + font.descent));

					let x0 = xy.x + g.bearing_x;
					let y1 = xy.y + g.bearing_y;
					let x1 = x0 + g.width;
					let y0 = y1 - g.height;

					self.textured_rect((x0, y0, x1, y1), &font.atlas, g.uv, color);
					pos.x += g.advance;
				}
			}
		}
		self
	}

	pub fn finish(&self) -> gpu::Result<(Buffer<PainterVertex>, Buffer<u32>)> {
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
