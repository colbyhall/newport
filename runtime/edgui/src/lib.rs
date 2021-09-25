#![feature(trait_alias)]

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

#[derive(Clone, Copy, Debug)]
pub enum Alignment {
	Min,
	Center,
	Max,
}

impl Alignment {
	pub const LEFT: Alignment = Alignment::Min;
	pub const RIGHT: Alignment = Alignment::Max;

	pub const TOP: Alignment = Alignment::Min;
	pub const BOTTOM: Alignment = Alignment::Max;

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
}
