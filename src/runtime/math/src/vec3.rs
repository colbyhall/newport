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

use crate::lerp;
use crate::Vec2;

use serde::{
	Deserialize,
	Deserializer,
	Serialize,
	Serializer,
};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Vec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vec3 {
	pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
	pub const ONE: Self = Self::new(1.0, 1.0, 1.0);

	pub const FORWARD: Self = Self::new(1.0, 0.0, 0.0);
	pub const RIGHT: Self = Self::new(0.0, 1.0, 0.0);
	pub const UP: Self = Self::new(0.0, 0.0, 1.0);

	pub const fn new(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}

	pub const fn dot(self, other: Self) -> f32 {
		self.x * other.x + self.y * other.y * self.z * other.z
	}

	pub const fn cross(self, other: Self) -> Self {
		Self {
			x: self.y * other.z - other.y * self.z,
			y: self.z * other.x - other.z * self.x,
			z: self.x * other.y - other.x * self.y,
		}
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
		Self::new(self.x.abs(), self.y.abs(), self.z.abs())
	}

	pub const fn is_finite(self) -> bool {
		self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
	}

	pub const fn is_nan(self) -> bool {
		self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
	}

	pub const fn min(self, other: Self) -> Self {
		let x = if self.x < other.x { self.x } else { other.x };

		let y = if self.y < other.y { self.y } else { other.y };

		let z = if self.z < other.z { self.z } else { other.z };

		Self::new(x, y, z)
	}

	pub const fn max(self, other: Self) -> Self {
		let x = if self.x > other.x { self.x } else { other.x };

		let y = if self.y > other.y { self.y } else { other.y };

		let z = if self.z > other.z { self.z } else { other.z };

		Self::new(x, y, z)
	}

	pub const fn min_elem(self) -> f32 {
		if self.x < self.y {
			if self.x < self.z {
				self.x
			} else {
				self.z
			}
		} else if self.y < self.z {
			self.y
		} else {
			self.z
		}
	}

	pub const fn max_elem(self) -> f32 {
		if self.x > self.y {
			if self.x > self.z {
				self.x
			} else {
				self.z
			}
		} else if self.y > self.z {
			self.y
		} else {
			self.z
		}
	}

	pub fn lerp(a: Self, b: Self, t: f32) -> Self {
		Self::new(lerp(a.x, b.x, t), lerp(a.y, b.y, t), lerp(a.z, b.z, t))
	}
}

impl Add for Vec3 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl Add<f32> for Vec3 {
	type Output = Self;

	fn add(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x + rhs,
			y: self.y + rhs,
			z: self.z + rhs,
		}
	}
}

impl AddAssign for Vec3 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
	}
}

impl AddAssign<f32> for Vec3 {
	fn add_assign(&mut self, rhs: f32) {
		self.x += rhs;
		self.y += rhs;
		self.z += rhs;
	}
}

impl Sub for Vec3 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl Sub<f32> for Vec3 {
	type Output = Self;

	fn sub(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x - rhs,
			y: self.y - rhs,
			z: self.z - rhs,
		}
	}
}

impl SubAssign for Vec3 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
	}
}

impl SubAssign<f32> for Vec3 {
	fn sub_assign(&mut self, rhs: f32) {
		self.x -= rhs;
		self.y -= rhs;
		self.z -= rhs;
	}
}

impl Mul for Vec3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
		}
	}
}

impl Mul<f32> for Vec3 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
		}
	}
}

impl MulAssign for Vec3 {
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
	}
}

impl MulAssign<f32> for Vec3 {
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
	}
}

impl Div for Vec3 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
			z: self.z / rhs.z,
		}
	}
}

impl Div<f32> for Vec3 {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
		}
	}
}

impl DivAssign for Vec3 {
	fn div_assign(&mut self, rhs: Self) {
		self.x /= rhs.x;
		self.y /= rhs.y;
		self.z /= rhs.z;
	}
}

impl DivAssign<f32> for Vec3 {
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
	}
}

impl Neg for Vec3 {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
			z: -self.z,
		}
	}
}

impl From<(Vec2, f32)> for Vec3 {
	fn from(v: (Vec2, f32)) -> Self {
		let (xy, z) = v;
		Self::new(xy.x, xy.y, z)
	}
}

impl From<[f32; 3]> for Vec3 {
	fn from(xyz: [f32; 3]) -> Self {
		Self::new(xyz[0], xyz[1], xyz[2])
	}
}

impl Serialize for Vec3 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let xyz = [self.x, self.y, self.z];
		xyz.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for Vec3 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let xyz = <[f32; 3]>::deserialize(deserializer)?;
		Ok(Vec3 {
			x: xyz[0],
			y: xyz[1],
			z: xyz[2],
		})
	}
}
