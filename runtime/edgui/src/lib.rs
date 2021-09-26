#![feature(trait_alias)]

use std::ops::RangeInclusive;

use math::{
	Rect,
	Vector2,
};

mod context;
mod gruvbox;
mod gui;
mod id;
mod input;
mod paint;
mod placement;
mod response;
mod retained;
mod sense;
mod style;
mod widgets;

pub use {
	context::*,
	gruvbox::*,
	gui::*,
	id::*,
	input::*,
	paint::*,
	placement::*,
	response::*,
	retained::*,
	sense::*,
	style::*,
	widgets::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
	Min,
	Center,
	Max,
}

impl Alignment {
	pub const LEFT: Alignment = Alignment::Min;
	pub const RIGHT: Alignment = Alignment::Max;

	pub const BOTTOM: Alignment = Alignment::Min;
	pub const TOP: Alignment = Alignment::Max;

	pub fn to_factor(self) -> f32 {
		match self {
			Alignment::Min => 0.0,
			Alignment::Center => 0.5,
			Alignment::Max => 1.0,
		}
	}

	pub fn to_sign(self) -> f32 {
		match self {
			Alignment::Min => -1.0,
			Alignment::Center => 0.0,
			Alignment::Max => 1.0,
		}
	}

	pub fn align_size_within_range(
		self,
		size: f32,
		range: RangeInclusive<f32>,
	) -> RangeInclusive<f32> {
		let min = *range.start();
		let max = *range.end();

		if max - min == f32::INFINITY && size == f32::INFINITY {
			return range;
		}

		match self {
			Self::Min => min..=min + size,
			Self::Center => {
				if size == f32::INFINITY {
					f32::NEG_INFINITY..=f32::INFINITY
				} else {
					let left = (min + max) / 2.0 - size / 2.0;
					left..=left + size
				}
			}
			Self::Max => max - size..=max,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alignment2d(pub [Alignment; 2]);

impl Alignment2d {
	pub const LEFT_BOTTOM: Alignment2d = Alignment2d([Alignment::Min, Alignment::Max]);
	pub const LEFT_CENTER: Alignment2d = Alignment2d([Alignment::Min, Alignment::Center]);
	pub const LEFT_TOP: Alignment2d = Alignment2d([Alignment::Min, Alignment::Min]);
	pub const CENTER_BOTTOM: Alignment2d = Alignment2d([Alignment::Center, Alignment::Max]);
	pub const CENTER_CENTER: Alignment2d = Alignment2d([Alignment::Center, Alignment::Center]);
	pub const CENTER_TOP: Alignment2d = Alignment2d([Alignment::Center, Alignment::Min]);
	pub const RIGHT_BOTTOM: Alignment2d = Alignment2d([Alignment::Max, Alignment::Max]);
	pub const RIGHT_CENTER: Alignment2d = Alignment2d([Alignment::Max, Alignment::Center]);
	pub const RIGHT_TOP: Alignment2d = Alignment2d([Alignment::Max, Alignment::Min]);

	pub fn x(self) -> Alignment {
		self.0[0]
	}

	pub fn y(self) -> Alignment {
		self.0[1]
	}

	pub fn to_sign(self) -> Vector2 {
		(self.x().to_sign(), self.y().to_sign()).into()
	}

	pub fn align_size_within_rect(self, size: Vector2, frame: Rect) -> Rect {
		let x_range = self.x().align_size_within_range(size.x, frame.x_range());
		let y_range = self.y().align_size_within_range(size.y, frame.y_range());
		Rect::from_ranges(x_range, y_range)
	}
}
