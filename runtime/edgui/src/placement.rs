use math::{
	Rect,
	Vector2,
};

use crate::{
	Alignment,
	Alignment2d,
};

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
	wrap: bool,
	alignment: Alignment,
	justify: bool,
	cross_alignment: Alignment,
	cross_justify: bool,
}

impl Layout {
	pub fn new(direction: Direction) -> Self {
		Self {
			direction,
			wrap: false,
			alignment: Alignment::Center,
			justify: false,
			cross_alignment: Alignment::Center,
			cross_justify: false,
		}
	}

	pub fn top_to_bottom() -> Self {
		Self {
			direction: Direction::TopToBottom,
			wrap: false,
			alignment: Alignment::Center,
			justify: false,
			cross_alignment: Alignment::Center,
			cross_justify: false,
		}
	}

	pub fn bottom_to_top() -> Self {
		Self {
			direction: Direction::BottomToTop,
			wrap: false,
			alignment: Alignment::Center,
			justify: false,
			cross_alignment: Alignment::Center,
			cross_justify: false,
		}
	}

	pub fn left_to_right() -> Self {
		Self {
			direction: Direction::LeftToRight,
			wrap: false,
			alignment: Alignment::Center,
			justify: false,
			cross_alignment: Alignment::Center,
			cross_justify: false,
		}
	}

	pub fn right_to_left() -> Self {
		Self {
			direction: Direction::RightToLeft,
			wrap: false,
			alignment: Alignment::Center,
			justify: false,
			cross_alignment: Alignment::Center,
			cross_justify: false,
		}
	}
}

impl Layout {
	pub fn direction(&self) -> Direction {
		self.direction
	}

	pub fn wrap(&self) -> bool {
		self.wrap
	}

	pub fn alignment(&self) -> Alignment {
		self.alignment
	}

	pub fn justify(&self) -> bool {
		self.justify
	}

	pub fn cross_alignment(&self) -> Alignment {
		self.cross_alignment
	}

	pub fn cross_justify(&self) -> bool {
		self.cross_justify
	}

	pub fn is_vertical(&self) -> bool {
		match self.direction {
			Direction::TopToBottom | Direction::BottomToTop => true,
			_ => false,
		}
	}

	pub fn is_horizontal(&self) -> bool {
		match self.direction {
			Direction::LeftToRight | Direction::RightToLeft => true,
			_ => false,
		}
	}

	pub fn horizontal_align(&self) -> Alignment {
		if self.is_horizontal() {
			self.alignment
		} else {
			self.cross_alignment
		}
	}

	pub fn vertical_align(&self) -> Alignment {
		if self.is_vertical() {
			self.alignment
		} else {
			self.cross_alignment
		}
	}

	pub fn alignment2d(&self) -> Alignment2d {
		Alignment2d([self.alignment, self.cross_alignment])
	}

	pub fn horizontal_justify(&self) -> bool {
		if self.is_horizontal() {
			self.justify
		} else {
			self.cross_justify
		}
	}

	pub fn vertical_justify(&self) -> bool {
		if self.is_vertical() {
			self.justify
		} else {
			self.cross_justify
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct Placer {
	pub bounds: Rect,
	pub used: Rect,
	pub cursor: Rect,

	pub layout: Layout,
}

impl Placer {
	pub fn layout(bounds: Rect, layout: Layout) -> Self {
		let mut cursor = bounds;

		match layout.direction {
			Direction::LeftToRight => {
				cursor.max.x = f32::INFINITY;
			}
			Direction::RightToLeft => {
				cursor.min.x = -f32::INFINITY;
			}
			Direction::TopToBottom => {
				cursor.max.y = f32::INFINITY;
			}
			Direction::BottomToTop => {
				cursor.min.y = -f32::INFINITY;
			}
		}

		let mut result = Self {
			bounds,
			used: Rect::ZERO, // temp
			cursor,

			layout,
		};
		let seed = result.next_widget_position();
		result.used = Rect::from_center(seed, Vector2::ZERO);
		result
	}

	pub fn available_rect_before_wrap(&self) -> Rect {
		let mut avail = self.bounds;

		match self.layout.direction {
			Direction::LeftToRight => {
				avail.min.x = self.cursor.min.x;
				avail.max.x = avail.max.x.max(self.cursor.min.x);
				avail.max.x = avail.max.x.max(avail.min.x);
				avail.max.y = avail.max.y.max(avail.min.y);
			}
			Direction::RightToLeft => {
				avail.max.x = self.cursor.max.x;
				avail.min.x = avail.min.x.min(self.cursor.max.x);
				avail.min.x = avail.min.x.min(avail.max.x);
				avail.max.y = avail.max.y.max(avail.min.y);
			}
			Direction::TopToBottom => {
				avail.min.y = self.cursor.min.y;
				avail.max.y = avail.max.y.max(self.cursor.min.y);
				avail.max.x = avail.max.x.max(avail.min.x);
				avail.max.y = avail.max.y.max(avail.min.y);
			}
			Direction::BottomToTop => {
				avail.max.y = self.cursor.max.y;
				avail.min.y = avail.min.y.min(self.cursor.max.y);
				avail.max.x = avail.max.x.max(avail.min.x);
				avail.min.y = avail.min.y.min(avail.max.y);
			}
		}

		// We can use the cursor to restrict the available region.
		// For instance, we use this to restrict the available space of a parent Ui
		// after adding a panel to it.
		// We also use it for wrapping layouts.
		avail = avail.intersect(self.cursor);

		// Make sure it isn't negative:
		if avail.max.x < avail.min.x {
			let x = 0.5 * (avail.min.x + avail.max.x);
			avail.min.x = x;
			avail.max.x = x;
		}
		if avail.max.y < avail.min.y {
			let y = 0.5 * (avail.min.y + avail.max.y);
			avail.min.y = y;
			avail.max.y = y;
		}

		avail
	}

	fn next_frame_ignore_wrap(&self, child_size: Vector2) -> Rect {
		let available_rect = self.available_rect_before_wrap();

		let mut frame_size = child_size;

		if (self.layout.is_vertical() && self.layout.horizontal_align() == Alignment::Center)
			|| self.layout.horizontal_justify()
		{
			frame_size.x = frame_size.x.max(available_rect.width()); // fill full width
		}
		if (self.layout.is_horizontal() && self.layout.vertical_align() == Alignment::Center)
			|| self.layout.vertical_justify()
		{
			frame_size.y = frame_size.y.max(available_rect.height()); // fill full height
		}

		let alignment = match self.layout.direction {
			Direction::LeftToRight => Alignment2d([Alignment::LEFT, self.layout.vertical_align()]),
			Direction::RightToLeft => Alignment2d([Alignment::RIGHT, self.layout.vertical_align()]),
			Direction::TopToBottom => Alignment2d([self.layout.horizontal_align(), Alignment::TOP]),
			Direction::BottomToTop => {
				Alignment2d([self.layout.horizontal_align(), Alignment::BOTTOM])
			}
		};

		let mut frame_rect = alignment.align_size_within_rect(frame_size, available_rect);

		if self.layout.is_horizontal() && frame_rect.max.y < self.cursor.max.y {
			// for horizontal layouts we always want to expand down,
			// or we will overlap the row above.
			// This is a bit hacky. Maybe we should do it for vertical layouts too.
			frame_rect = frame_rect.translate(Vector2::UP * (self.cursor.max.y - frame_rect.max.y));
		}

		frame_rect
	}

	pub(crate) fn next_widget_space_ignore_wrap_justify(&self, size: Vector2) -> Rect {
		let frame = self.next_frame_ignore_wrap(size);
		self.layout
			.alignment2d()
			.align_size_within_rect(size, frame)
	}

	pub(crate) fn next_widget_position(&self) -> Vector2 {
		self.next_widget_space_ignore_wrap_justify(Vector2::ZERO)
			.center()
	}

	pub(crate) fn advance_after_rects(
		&mut self,
		frame_rect: Rect,
		widget_rect: Rect,
		item_spacing: Vector2,
	) {
		if self.layout.wrap {
			if self.cursor.overlaps(frame_rect.shrink(1.0)) {
				// make row/column larger if necessary
				self.cursor = self.cursor.expand(frame_rect);
			} else {
				// this is a new row or column. We temporarily use NAN for what will be filled in later.
				match self.layout.direction {
					Direction::LeftToRight => {
						self.cursor = Rect::from_min_max(
							(f32::NAN, frame_rect.min.y),
							(f32::INFINITY, frame_rect.max.y),
						);
					}
					Direction::RightToLeft => {
						self.cursor = Rect::from_min_max(
							(-f32::INFINITY, frame_rect.min.y),
							(f32::NAN, frame_rect.max.y),
						);
					}
					Direction::BottomToTop => {
						self.cursor = Rect::from_min_max(
							(frame_rect.min.x, f32::NAN),
							(frame_rect.max.x, f32::INFINITY),
						);
					}
					Direction::TopToBottom => {
						self.cursor = Rect::from_min_max(
							(frame_rect.min.x, -f32::INFINITY),
							(frame_rect.max.x, f32::NAN),
						);
					}
				};
			}
		} else {
			// Make sure we also expand where we consider adding things (the cursor):
			if self.layout.is_horizontal() {
				self.cursor.min.y = self.cursor.min.y.min(frame_rect.min.y);
				self.cursor.max.y = self.cursor.max.y.max(frame_rect.max.y);
			} else {
				self.cursor.min.x = self.cursor.min.x.min(frame_rect.min.x);
				self.cursor.max.x = self.cursor.max.x.max(frame_rect.max.x);
			}
		}

		match self.layout.direction {
			Direction::LeftToRight => {
				self.cursor.min.x = widget_rect.max.x + item_spacing.x;
			}
			Direction::RightToLeft => {
				self.cursor.max.x = widget_rect.min.x - item_spacing.x;
			}
			Direction::BottomToTop => {
				self.cursor.min.y = widget_rect.max.y + item_spacing.y;
			}
			Direction::TopToBottom => {
				self.cursor.max.y = widget_rect.min.y - item_spacing.y;
			}
		};

		self.used.expand(frame_rect); // e.g. for centered layouts: pretend we used whole frame
	}

	pub(crate) fn next_frame(&self, child_size: Vector2, spacing: Vector2) -> Rect {
		if self.layout.wrap {
			let available_size = self.available_rect_before_wrap().size();

			let Placer {
				mut cursor,
				mut bounds,
				used,
				layout,
			} = *self;

			match self.layout.direction {
				Direction::LeftToRight => {
					if available_size.x < child_size.x && bounds.left() < cursor.left() {
						// New row
						let new_row_height = cursor.height().max(child_size.y);
						// let new_top = cursor.bottom() + spacing.y;
						let new_top = used.bottom() + spacing.y; // tighter packing
						cursor = Rect::from_min_max(
							(bounds.left(), new_top),
							(f32::INFINITY, new_top + new_row_height),
						);
						bounds.max.y = bounds.max.y.max(cursor.max.y);
					}
				}
				Direction::RightToLeft => {
					if available_size.x < child_size.x && cursor.right() < bounds.right() {
						// New row
						let new_row_height = cursor.height().max(child_size.y);
						// let new_top = cursor.bottom() + spacing.y;
						let new_top = used.bottom() + spacing.y; // tighter packing
						cursor = Rect::from_min_max(
							(-f32::INFINITY, new_top),
							(bounds.right(), new_top + new_row_height),
						);
						bounds.max.y = bounds.max.y.max(cursor.max.y);
					}
				}
				Direction::BottomToTop => {
					if available_size.y < child_size.y && bounds.bottom() < cursor.bottom() {
						// New column
						let new_col_width = cursor.width().max(child_size.x);
						cursor = Rect::from_min_max(
							(cursor.right() + spacing.x, bounds.bottom()),
							(cursor.right() + spacing.x + new_col_width, f32::INFINITY),
						);
						bounds.max.x = bounds.max.x.max(cursor.max.x);
					}
				}
				Direction::TopToBottom => {
					if available_size.y < child_size.y && cursor.top() < bounds.top() {
						// New column
						let new_col_width = cursor.width().max(child_size.x);
						cursor = Rect::from_min_max(
							(cursor.right() + spacing.x, -f32::INFINITY),
							(cursor.right() + spacing.x + new_col_width, bounds.top()),
						);
						bounds.max.x = bounds.max.x.max(cursor.max.x);
					}
				}
			}

			// Use the new cursor:
			let placer = Placer {
				bounds,
				used,
				cursor,

				layout,
			};

			placer.next_frame_ignore_wrap(child_size)
		} else {
			self.next_frame_ignore_wrap(child_size)
		}
	}

	pub(crate) fn advance_cursor(&mut self, amount: f32) {
		match self.layout.direction {
			Direction::LeftToRight => {
				self.cursor.min.x += amount;
				self.used.expand_to_include_x(self.cursor.min.x);
			}
			Direction::RightToLeft => {
				self.cursor.max.x -= amount;
				self.used.expand_to_include_x(self.cursor.max.x);
			}
			Direction::TopToBottom => {
				self.cursor.max.y -= amount;
				self.used.expand_to_include_y(self.cursor.min.y);
			}
			Direction::BottomToTop => {
				self.cursor.min.y += amount;
				self.used.expand_to_include_y(self.cursor.max.y);
			}
		}
	}

	pub fn next_space(&self, child_size: Vector2, spacing: Vector2) -> Rect {
		self.next_frame(child_size, spacing)
	}

	pub fn justify_and_align(&self, frame: Rect, mut child_size: Vector2) -> Rect {
		if self.layout.horizontal_justify() {
			child_size.x = child_size.x.max(frame.width()); // fill full width
		}
		if self.layout.vertical_justify() {
			child_size.y = child_size.y.max(frame.height()); // fill full height
		}
		self.layout
			.alignment2d()
			.align_size_within_rect(child_size, frame)
	}
}

// 	pub fn push_size(&mut self, size: Vector2) -> Rect {
// 		match &self.layout.direction {
// 			Direction::LeftToRight => {
// 				let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
// 				self.cursor += size.x;
// 				let max = (self.bounds.min.x + self.cursor, self.bounds.max.y).into();
// 				Rect::from_min_max(min, max)
// 			}
// 			Direction::RightToLeft => {
// 				let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
// 				self.cursor += size.x;
// 				let min = (self.bounds.max.x - self.cursor, self.bounds.min.y).into();
// 				Rect::from_min_max(min, max)
// 			}
// 			Direction::TopToBottom => {
// 				let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
// 				self.cursor += size.y;
// 				let min = (self.bounds.min.x, self.bounds.max.y - self.cursor).into();
// 				Rect::from_min_max(min, max)
// 			}
// 			Direction::BottomToTop => {
// 				let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
// 				self.cursor += size.y;
// 				let max = (self.bounds.max.x, self.bounds.min.y + self.cursor).into();
// 				Rect::from_min_max(min, max)
// 			}
// 		}
// 	}

// 	pub fn space_left(&self) -> Vector2 {
// 		match &self.layout.direction {
// 			Direction::LeftToRight | Direction::RightToLeft => {
// 				(self.bounds.width() - self.cursor, self.bounds.height()).into()
// 			}
// 			Direction::TopToBottom | Direction::BottomToTop => {
// 				(self.bounds.width(), self.bounds.height() - self.cursor).into()
// 			}
// 		}
// 	}

// 	pub fn bounds(&self) -> Rect {
// 		self.bounds
// 	}

// 	pub fn available_rect(&self) -> Rect {
// 		match &self.layout.direction {
// 			Direction::LeftToRight => {
// 				let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
// 				let max = (
// 					self.bounds.min.x + self.cursor + self.space_left().x,
// 					self.bounds.max.y,
// 				)
// 					.into();
// 				Rect::from_min_max(min, max)
// 			}
// 			Direction::RightToLeft => {
// 				let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
// 				let min = (
// 					self.bounds.max.x - (self.cursor + self.space_left().x),
// 					self.bounds.min.y,
// 				)
// 					.into();
// 				Rect::from_min_max(min, max)
// 			}
// 			Direction::TopToBottom => {
// 				let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
// 				let min = (
// 					self.bounds.min.x,
// 					self.bounds.max.y - (self.cursor + self.space_left().y),
// 				)
// 					.into();
// 				Rect::from_min_max(min, max)
// 			}
// 			Direction::BottomToTop => {
// 				let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
// 				let max = (
// 					self.bounds.max.x,
// 					self.bounds.min.y + (self.cursor + self.space_left().y),
// 				)
// 					.into();
// 				Rect::from_min_max(min, max)
// 			}
// 		}
// 	}
// }
