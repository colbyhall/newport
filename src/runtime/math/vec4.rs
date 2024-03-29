use std::convert::From;
use std::ops::Index;
use std::ops::IndexMut;
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

use crate::Rect;
use crate::Vec3;

use serde::{
	Deserialize,
	Deserializer,
	Serialize,
	Serializer,
};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Vec4 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

impl Vec4 {
	pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

	pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
		Self { x, y, z, w }
	}

	pub const fn dot(self, rhs: Self) -> f32 {
		self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
	}

	pub const fn to_tuple(self) -> (f32, f32, f32, f32) {
		(self.x, self.y, self.z, self.w)
	}

	pub const fn xyz(self) -> Vec3 {
		Vec3::new(self.x, self.y, self.z)
	}
}

impl Index<usize> for Vec4 {
	type Output = f32;

	fn index(&self, index: usize) -> &Self::Output {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			3 => &self.w,
			_ => unreachable!(),
		}
	}
}

impl IndexMut<usize> for Vec4 {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		match index {
			0 => &mut self.x,
			1 => &mut self.y,
			2 => &mut self.z,
			3 => &mut self.w,
			_ => unreachable!(),
		}
	}
}

impl Add for Vec4 {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
			w: self.w + rhs.w,
		}
	}
}

impl Add<f32> for Vec4 {
	type Output = Self;

	fn add(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x + rhs,
			y: self.y + rhs,
			z: self.z + rhs,
			w: self.w + rhs,
		}
	}
}

impl AddAssign for Vec4 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
		self.w += rhs.w;
	}
}

impl AddAssign<f32> for Vec4 {
	fn add_assign(&mut self, rhs: f32) {
		self.x += rhs;
		self.y += rhs;
		self.z += rhs;
		self.w += rhs;
	}
}

impl Sub for Vec4 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
			w: self.w - rhs.w,
		}
	}
}

impl Sub<f32> for Vec4 {
	type Output = Self;

	fn sub(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x - rhs,
			y: self.y - rhs,
			z: self.z - rhs,
			w: self.w - rhs,
		}
	}
}

impl SubAssign for Vec4 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
		self.w -= rhs.w;
	}
}

impl SubAssign<f32> for Vec4 {
	fn sub_assign(&mut self, rhs: f32) {
		self.x -= rhs;
		self.y -= rhs;
		self.z -= rhs;
		self.w -= rhs;
	}
}

impl Mul for Vec4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x * rhs.x,
			y: self.y * rhs.y,
			z: self.z * rhs.z,
			w: self.w * rhs.w,
		}
	}
}

impl Mul<f32> for Vec4 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x * rhs,
			y: self.y * rhs,
			z: self.z * rhs,
			w: self.w * rhs,
		}
	}
}

impl MulAssign for Vec4 {
	fn mul_assign(&mut self, rhs: Self) {
		self.x *= rhs.x;
		self.y *= rhs.y;
		self.z *= rhs.z;
		self.z *= rhs.z;
	}
}

impl MulAssign<f32> for Vec4 {
	fn mul_assign(&mut self, rhs: f32) {
		self.x *= rhs;
		self.y *= rhs;
		self.z *= rhs;
		self.z *= rhs;
	}
}

impl Div for Vec4 {
	type Output = Self;

	fn div(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x / rhs.x,
			y: self.y / rhs.y,
			z: self.z / rhs.z,
			w: self.w / rhs.w,
		}
	}
}

impl Div<f32> for Vec4 {
	type Output = Self;

	fn div(self, rhs: f32) -> Self::Output {
		Self {
			x: self.x / rhs,
			y: self.y / rhs,
			z: self.z / rhs,
			w: self.w / rhs,
		}
	}
}

impl DivAssign for Vec4 {
	fn div_assign(&mut self, rhs: Self) {
		self.x /= rhs.x;
		self.y /= rhs.y;
		self.z /= rhs.z;
		self.w /= rhs.w;
	}
}

impl DivAssign<f32> for Vec4 {
	fn div_assign(&mut self, rhs: f32) {
		self.x /= rhs;
		self.y /= rhs;
		self.z /= rhs;
		self.w /= rhs;
	}
}

impl Neg for Vec4 {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
			z: -self.z,
			w: -self.w,
		}
	}
}

impl From<f32> for Vec4 {
	fn from(s: f32) -> Self {
		Vec4::new(s, s, s, s)
	}
}

impl From<(Vec3, f32)> for Vec4 {
	fn from(xyzw: (Vec3, f32)) -> Self {
		let (xyz, w) = xyzw;
		Vec4::new(xyz.x, xyz.y, xyz.z, w)
	}
}

impl From<Rect> for Vec4 {
	fn from(rect: Rect) -> Self {
		Self {
			x: rect.min.x,
			y: rect.min.y,
			z: rect.max.x,
			w: rect.max.y,
		}
	}
}

impl Serialize for Vec4 {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let xyzw = [self.x, self.y, self.z, self.w];
		xyzw.serialize(serializer)
	}
}

impl<'de> Deserialize<'de> for Vec4 {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let xyzw = <[f32; 4]>::deserialize(deserializer)?;
		Ok(Vec4 {
			x: xyzw[0],
			y: xyzw[1],
			z: xyzw[2],
			w: xyzw[3],
		})
	}
}
