use std::ops::Index;
use std::ops::IndexMut;
use std::ops::{
	Mul,
	MulAssign,
};

use crate::Vec2;
use crate::Vec3;

use serde::{
	Deserialize,
	Serialize,
};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Mat3 {
	pub x_column: Vec3,
	pub y_column: Vec3,
	pub z_column: Vec3,
}

impl Default for Mat3 {
	fn default() -> Self {
		Mat3::IDENTITY
	}
}

impl Mat3 {
	pub const ZERO: Self = Self {
		x_column: Vec3::ZERO,
		y_column: Vec3::ZERO,
		z_column: Vec3::ZERO,
	};

	pub const IDENTITY: Self = Self {
		x_column: Vec3::new(1.0, 0.0, 0.0),
		y_column: Vec3::new(0.0, 1.0, 0.0),
		z_column: Vec3::new(0.0, 0.0, 1.0),
	};

	pub const fn from_cols(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Self {
		Self {
			x_column: x_axis,
			y_column: y_axis,
			z_column: z_axis,
		}
	}

	pub const fn from_rows(x_axis: Vec3, y_axis: Vec3, z_axis: Vec3) -> Self {
		let x = Vec3::new(x_axis.x, y_axis.x, z_axis.x);
		let y = Vec3::new(x_axis.y, y_axis.y, z_axis.y);
		let z = Vec3::new(x_axis.z, y_axis.z, z_axis.z);
		Self::from_cols(x, y, z)
	}

	pub fn col(&self, index: usize) -> Vec3 {
		match index {
			0 => self.x_column,
			1 => self.y_column,
			2 => self.z_column,
			_ => Vec3::ZERO,
		}
	}

	pub fn row(&self, index: usize) -> Vec3 {
		match index {
			0 => Vec3::new(self.x_column.x, self.y_column.x, self.z_column.x),
			1 => Vec3::new(self.x_column.y, self.y_column.y, self.z_column.y),
			2 => Vec3::new(self.x_column.z, self.y_column.z, self.z_column.z),
			_ => Vec3::ZERO,
		}
	}

	pub fn translate(xy: impl Into<Vec2>) -> Self {
		let mut result = Self::IDENTITY;
		result.z_column = (xy.into(), 1.0).into();
		result
	}

	pub fn rotate(rads: f32) -> Self {
		let x_axis = Vec2::from_rad(rads);
		let y_axis = x_axis.perp();

		let mut result = Self::IDENTITY;
		result.x_column = (x_axis, 0.0).into();
		result.y_column = (y_axis, 0.0).into();
		result
	}

	pub fn scale(xy: impl Into<Vec2>) -> Self {
		let xy = xy.into();
		Self {
			x_column: Vec3::new(xy.x, 0.0, 0.0),
			y_column: Vec3::new(0.0, xy.y, 0.0),
			z_column: Vec3::new(0.0, 0.0, 1.0),
		}
	}

	pub fn inverse(self) -> Option<Self> {
		unimplemented!()
	}
}

impl Index<usize> for Mat3 {
	type Output = f32;

	fn index(&self, index: usize) -> &Self::Output {
		let col = index / 3;
		let row = index % 3;

		let col = match col {
			0 => (&self.x_column),
			1 => (&self.y_column),
			2 => (&self.z_column),
			_ => unreachable!(),
		};

		&col[row]
	}
}

impl IndexMut<usize> for Mat3 {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		let col = index / 3;
		let row = index % 3;

		let col = match col {
			0 => (&mut self.x_column),
			1 => (&mut self.y_column),
			2 => (&mut self.z_column),
			_ => unreachable!(),
		};

		&mut col[row]
	}
}

impl Mul for Mat3 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut row_x = Vec3::ZERO;
		row_x.x = self.row(0).dot(rhs.x_column);
		row_x.y = self.row(0).dot(rhs.y_column);
		row_x.z = self.row(0).dot(rhs.z_column);

		let mut row_y = Vec3::ZERO;
		row_y.x = self.row(1).dot(rhs.x_column);
		row_y.y = self.row(1).dot(rhs.y_column);
		row_y.z = self.row(1).dot(rhs.z_column);

		let mut row_z = Vec3::ZERO;
		row_z.x = self.row(2).dot(rhs.x_column);
		row_z.y = self.row(2).dot(rhs.y_column);
		row_z.z = self.row(2).dot(rhs.z_column);

		Self::from_rows(row_x, row_y, row_z)
	}
}

impl Mul<Vec3> for Mat3 {
	type Output = Vec3;

	fn mul(self, rhs: Vec3) -> Self::Output {
		let x = self.row(0).dot(rhs);
		let y = self.row(1).dot(rhs);
		let z = self.row(2).dot(rhs);

		Vec3::new(x, y, z)
	}
}

impl MulAssign for Mat3 {
	fn mul_assign(&mut self, rhs: Self) {
		*self = *self * rhs;
	}
}
