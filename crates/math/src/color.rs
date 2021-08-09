use crate::Vector4;
use std::convert::From;

use serde::{
	Deserialize,
	Serialize,
};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

impl Color {
	pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
	pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
	pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);

	pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
	pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

	pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
	pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
	pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);

	pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self {
			r: r,
			g: g,
			b: b,
			a: a,
		}
	}

	pub const fn from_hex(hex: u32) -> Self {
		let r = hex >> 24 & 0xFF;
		let g = hex >> 16 & 0xFF;
		let b = hex >> 8 & 0xFF;
		let a = hex & 0xFF;

		Self {
			r: r as f32 / 255.0,
			g: g as f32 / 255.0,
			b: b as f32 / 255.0,
			a: a as f32 / 255.0,
		}
	}

	pub fn from_srgb(hex: u32) -> Self {
		let r = hex >> 24 & 0xFF;
		let g = hex >> 16 & 0xFF;
		let b = hex >> 8 & 0xFF;
		let a = hex & 0xFF;

		Self {
			r: srgb_to_linear(r as f32 / 255.0),
			g: srgb_to_linear(g as f32 / 255.0),
			b: srgb_to_linear(b as f32 / 255.0),
			a: srgb_to_linear(a as f32 / 255.0),
		}
	}
}

impl From<u32> for Color {
	fn from(color: u32) -> Self {
		Color::from_hex(color)
	}
}

impl From<Vector4> for Color {
	fn from(color: Vector4) -> Self {
		Self {
			r: color.x,
			g: color.y,
			b: color.z,
			a: color.w,
		}
	}
}

impl From<(f32, f32, f32, f32)> for Color {
	fn from(rgba: (f32, f32, f32, f32)) -> Self {
		let (r, g, b, a) = rgba;
		Self { r, g, b, a }
	}
}

pub fn linear_to_srgb(x: f32) -> f32 {
	if x <= 0.0 {
		return 0.0;
	} else if x >= 1.0 {
		return 1.0;
	} else if x < 0.0031308 {
		return x * 12.92;
	} else {
		return x.powf(1.0 / 2.4) * 1.055 - 0.055;
	}
}

pub fn srgb_to_linear(x: f32) -> f32 {
	if x <= 0.0 {
		return 0.0;
	} else if x >= 1.0 {
		return 1.0;
	} else if x <= 0.04045 {
		return x / 12.92;
	} else {
		return ((x + 0.055) / 1.055).powf(2.4);
	}
}
