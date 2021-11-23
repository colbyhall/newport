use std::ops::{
	Add,
	AddAssign,

	Div,
	DivAssign,

	Mul,
	MulAssign,

	Neg,
	Sub,
	SubAssign,
};

use std::convert::From;

use serde::{
	Deserialize,
	Deserializer,
	Serialize,
	Serializer,
};

use crate::lerp;

#[macro_export]
macro_rules! vec2 {
	() => {
		$crate::Vector2::ZERO
	};
	($xy:expr) => {
		$crate::Vector2::splat($xy)
	};
	($x:expr, $y:expr) => {
		$crate::Vector2::new($x, $y)
	};
}

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32,
}

impl Vector2 {
	pub const ZERO: Self = Self::new(0.0, 0.0);
	pub const ONE: Self = Self::new(1.0, 1.0);

	pub const RIGHT: Self = Self::new(1.0, 0.0);
	pub const UP: Self = Self::new(0.0, 1.0);

	pub const INFINITY: Self = Self::new(f32::INFINITY, f32::INFINITY);

	pub const fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	pub const fn splat(xy: f32) -> Self {
		Self { x: xy, y: xy }
	}

	pub fn from_rad(theta: f32) -> Self {
		Self {
			x: theta.sin(),
			y: theta.cos(),
		}
	}

	pub const fn dot(self, rhs: Self) -> f32 {
		self.x * rhs.x + self.y * rhs.y
	}

	pub const fn cross(self, rhs: Self) -> f32 {
		self.x * rhs.y - self.y * rhs.x
	}

	pub const fn perp(self) -> Self {
		Self::new(self.y, -self.x)
	}

	pub const fn len_sq(self) -> f32 {
		self.dot(self)
	}

	pub fn len(self) -> f32 {
		self.len_sq().sqrt()
	}

	pub fn is_empty(self) -> bool {
		self.len() < crate::SMALL_NUMBER
	}

	pub fn norm(self) -> Self {
		if self.is_empty() {
			Self::ZERO
		} else {
			self / self.len()
		}
	}

	pub fn abs(self) -> Self {
		Vector2::new(self.x.abs(), self.y.abs())
	}

	pub const fn is_finite(self) -> bool {
		self.x.is_finite() && self.y.is_finite()
	}

	pub const fn is_nan(self) -> bool {
		self.x.is_nan() || self.y.is_nan()
	}

	pub const fn min(self, other: Self) -> Self {
		let x = if self.x < other.x { self.x } else { other.x };

		let y = if self.y < other.y { self.y } else { other.y };

		Self::new(x, y)
	}

	pub const fn max(self, other: Self) -> Self {
		let x = if self.x > other.x { self.x } else { other.x };

		let y = if self.y > other.y { self.y } else { other.y };

		Self::new(x, y)
	}

	pub const fn min_elem(self) -> f32 {
		if self.x < self.y {
			self.x
		} else {
			self.y
		}
	}

	pub const fn max_elem(self) -> f32 {
		if self.x > self.y {
			self.x
		} else {
			self.y
		}
	}

	pub fn lerp(a: Self, b: Self, t: f32) -> Self {
		Self::new(lerp(a.x, b.x, t), lerp(a.y, b.y, t))
	}
}

impl Add for Vector2 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl Add<f32> for Vector2 {
	type Output = Self;

	fn add(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x + rhs,
			y: self.y + rhs,
		}
	}
}

impl AddAssign for Vector2 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl AddAssign<f32> for Vector2 {
	fn add_assign(&mut self, rhs: f32) {
		self.x += rhs;
		self.y += rhs;
	}
}

impl Sub for Vector2 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl Sub<f32> for Vector2 {
	type Output = Self;

	fn sub(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x - rhs,
			y: self.y - rhs,
		}
	}
}

impl SubAssign for Vector2 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl SubAssign<f32> for Vector2 {
	fn sub_assign(&mut self, rhs: f32) {
		self.x -= rhs;
		self.y -= rhs;
	}
}

impl Mul for Vector2 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
		}
	}
}

impl Mul<f32> for Vector2 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
		}
	}
}

impl MulAssign for Vector2 {
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
	}
}

impl MulAssign<f32> for Vector2 {
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
	}
}

impl Div for Vector2 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
		}
	}
}

impl Div<f32> for Vector2 {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
		}
	}
}

impl DivAssign for Vector2 {
	fn div_assign(&mut self, rhs: Self) {
		self.x /= rhs.x;
		self.y /= rhs.y;
	}
}

impl DivAssign<f32> for Vector2 {
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
	}
}

impl Neg for Vector2 {
	type Output = Self;
	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
		}
	}
}

impl From<(f32, f32)> for Vector2 {
	fn from(xy: (f32, f32)) -> Self {
		let (x, y) = xy;
		Self { x, y }
	}
}

impl From<[f32; 2]> for Vector2 {
	fn from(xy: [f32; 2]) -> Self {
		Self { x: xy[0], y: xy[1] }
	}
}

impl Serialize for Vector2 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let xy = [self.x, self.y];
		xy.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for Vector2 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let xy = <[f32; 2]>::deserialize(deserializer)?;
		Ok(Vector2 { x: xy[0], y: xy[1] })
	}
}
