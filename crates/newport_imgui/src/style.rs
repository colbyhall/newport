use crate::asset::AssetRef;
use crate::graphics::FontCollection;
use crate::math::{
	Color,
	Rect,
	Vector2,
};
use crate::DARK;

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
};

pub trait Style = Default + Clone + Any + 'static;

pub struct StyleMap {
	inner: HashMap<TypeId, Box<dyn Any>>,
}

impl StyleMap {
	pub fn new() -> Self {
		Self {
			inner: HashMap::with_capacity(16),
		}
	}

	pub fn get<T: Style>(&mut self) -> T {
		let id = TypeId::of::<T>();

		if !self.inner.contains_key(&id) {
			self.inner.insert(id, Box::new(vec![T::default()]));
		}
		self.inner
			.get(&id)
			.unwrap()
			.downcast_ref::<Vec<T>>()
			.unwrap()
			.last()
			.unwrap()
			.clone()
	}

	pub fn push<T: Style>(&mut self, t: T) {
		let id = TypeId::of::<T>();

		if !self.inner.contains_key(&id) {
			self.inner.insert(id, Box::new(vec![T::default()]));
		}
		self.inner
			.get_mut(&id)
			.unwrap()
			.downcast_mut::<Vec<T>>()
			.unwrap()
			.push(t);
	}

	pub fn pop<T: Style>(&mut self) {
		let id = TypeId::of::<T>();

		if !self.inner.contains_key(&id) {
			self.inner.insert(id, Box::new(vec![T::default()]));
			return;
		}

		let vec = self
			.inner
			.get_mut(&id)
			.unwrap()
			.downcast_mut::<Vec<T>>()
			.unwrap();
		vec.pop();
	}
}

#[derive(Clone, Copy)]
pub enum Sizing {
	MinMax { min: f32, max: f32 },
	Fill,
}

#[derive(Clone, Copy)]
pub struct LayoutStyle {
	pub margin: Rect,
	pub padding: Rect,

	pub width_sizing: Sizing,
	pub height_sizing: Sizing,
}

impl Default for LayoutStyle {
	fn default() -> Self {
		Self {
			margin: (5.0, 5.0, 5.0, 5.0).into(),
			padding: (5.0, 5.0, 5.0, 5.0).into(),

			width_sizing: Sizing::MinMax {
				min: 0.0,
				max: f32::INFINITY,
			},
			height_sizing: Sizing::MinMax {
				min: 0.0,
				max: f32::INFINITY,
			},
		}
	}
}

impl LayoutStyle {
	pub fn content_size(&self, mut needed: Vector2, available: Vector2) -> Vector2 {
		needed += self.padding.min + self.padding.max;

		let width = match self.width_sizing {
			Sizing::MinMax { min, max } => needed.x.max(min).min(max),
			Sizing::Fill => available.x,
		};

		let height = match self.height_sizing {
			Sizing::MinMax { min, max } => needed.y.max(min).min(max),
			Sizing::Fill => available.y,
		};

		(width, height).into()
	}

	pub fn spacing_size(&self, size: Vector2) -> Vector2 {
		size + self.margin.min + self.margin.max
	}
}

#[derive(Clone)]
pub enum Alignment {
	Left,
	Center,
	Right,
}

#[derive(Clone)]
pub struct TextStyle {
	pub font: AssetRef<FontCollection>,
	pub alignment: Alignment,

	pub label_size: u32,
	pub header_size: u32,
}

impl Default for TextStyle {
	fn default() -> Self {
		Self {
			font: AssetRef::new("{cdb5cd33-004d-4518-ab20-93475b735cfa}").unwrap(),
			alignment: Alignment::Center,

			label_size: 10,
			header_size: 14,
		}
	}
}

impl TextStyle {
	pub fn string_rect(&self, string: &str, size: u32, wrap: Option<f32>) -> Rect {
		let font = self.font.font_at_size(size, 1.0).unwrap(); // NOTE: I don't think DPI matters here

		font.string_rect(string, wrap.unwrap_or(f32::INFINITY))
	}

	pub fn label_height(&self) -> f32 {
		let font = self.font.font_at_size(self.label_size, 1.0).unwrap(); // NOTE: I don't think DPI matters here
		font.ascent - font.descent
	}
}

#[derive(Clone)]
pub struct ColorStyle {
	pub inactive_background: Color,
	pub inactive_foreground: Color,

	pub unhovered_background: Color,
	pub unhovered_foreground: Color,

	pub hovered_background: Color,
	pub hovered_foreground: Color,

	pub focused_background: Color,
	pub focused_foreground: Color,

	pub selected_background: Color,
	pub selected_foreground: Color,
}

impl Default for ColorStyle {
	fn default() -> Self {
		Self {
			inactive_background: DARK.bg1,
			inactive_foreground: DARK.fg,

			unhovered_background: DARK.bg,
			unhovered_foreground: DARK.fg,

			hovered_background: DARK.bg2,
			hovered_foreground: DARK.fg2,

			focused_background: DARK.bg2,
			focused_foreground: DARK.fg3,

			selected_background: DARK.yellow1,
			selected_foreground: DARK.bg1,
		}
	}
}
