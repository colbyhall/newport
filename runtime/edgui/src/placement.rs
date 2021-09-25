use math::{
	Rect,
	Vector2,
};

// use crate::Alignment;

#[derive(Copy, Clone, Debug)]
pub enum Direction {
	LeftToRight,
	RightToLeft,
	TopToBottom,
	BottomToTop,
}

#[derive(Copy, Clone, Debug)]
pub struct Layout {
	direction: Direction,
	// wrap: bool,
	// alignment: Alignment,
	// justify: bool,
	// cross_alignment: Alignment,
	// cross_justify: bool,
}

impl Layout {
	pub fn new(direction: Direction) -> Self {
		Self { direction }
	}

	pub fn top_to_bottom() -> Layout {
		Layout {
			direction: Direction::TopToBottom,
		}
	}

	pub fn bottom_to_top() -> Layout {
		Layout {
			direction: Direction::BottomToTop,
		}
	}

	pub fn left_to_right() -> Layout {
		Layout {
			direction: Direction::LeftToRight,
		}
	}

	pub fn right_to_left() -> Layout {
		Layout {
			direction: Direction::RightToLeft,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Placer {
	pub bounds: Rect,
	pub layout: Layout,
	pub cursor: f32,
}

impl Placer {
	pub fn new(bounds: Rect, layout: Layout) -> Self {
		Self {
			bounds,
			layout,
			cursor: 0.0,
		}
	}

	pub fn push_size(&mut self, size: Vector2) -> Rect {
		match &self.layout.direction {
			Direction::LeftToRight => {
				let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
				self.cursor += size.x;
				let max = (self.bounds.min.x + self.cursor, self.bounds.max.y).into();
				Rect::from_min_max(min, max)
			}
			Direction::RightToLeft => {
				let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
				self.cursor += size.x;
				let min = (self.bounds.max.x - self.cursor, self.bounds.min.y).into();
				Rect::from_min_max(min, max)
			}
			Direction::TopToBottom => {
				let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
				self.cursor += size.y;
				let min = (self.bounds.min.x, self.bounds.max.y - self.cursor).into();
				Rect::from_min_max(min, max)
			}
			Direction::BottomToTop => {
				let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
				self.cursor += size.y;
				let max = (self.bounds.max.x, self.bounds.min.y + self.cursor).into();
				Rect::from_min_max(min, max)
			}
		}
	}

	pub fn space_left(&self) -> Vector2 {
		match &self.layout.direction {
			Direction::LeftToRight | Direction::RightToLeft => {
				(self.bounds.width() - self.cursor, self.bounds.height()).into()
			}
			Direction::TopToBottom | Direction::BottomToTop => {
				(self.bounds.width(), self.bounds.height() - self.cursor).into()
			}
		}
	}

	pub fn bounds(&self) -> Rect {
		self.bounds
	}

	pub fn available_rect(&self) -> Rect {
		match &self.layout.direction {
			Direction::LeftToRight => {
				let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
				let max = (
					self.bounds.min.x + self.cursor + self.space_left().x,
					self.bounds.max.y,
				)
					.into();
				Rect::from_min_max(min, max)
			}
			Direction::RightToLeft => {
				let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
				let min = (
					self.bounds.max.x - (self.cursor + self.space_left().x),
					self.bounds.min.y,
				)
					.into();
				Rect::from_min_max(min, max)
			}
			Direction::TopToBottom => {
				let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
				let min = (
					self.bounds.min.x,
					self.bounds.max.y - (self.cursor + self.space_left().y),
				)
					.into();
				Rect::from_min_max(min, max)
			}
			Direction::BottomToTop => {
				let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
				let max = (
					self.bounds.max.x,
					self.bounds.min.y + (self.cursor + self.space_left().y),
				)
					.into();
				Rect::from_min_max(min, max)
			}
		}
	}

	pub fn cursor(&self) -> f32 {
		self.cursor
	}
}
