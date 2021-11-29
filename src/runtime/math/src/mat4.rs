use std::ops::Index;
use std::ops::IndexMut;
use std::ops::{Mul, MulAssign};

use crate::Quaternion;
use crate::Vector3;
use crate::Vector4;
use crate::PI;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Matrix4 {
	pub x_column: Vector4,
	pub y_column: Vector4,
	pub z_column: Vector4,
	pub w_column: Vector4,
}

impl Default for Matrix4 {
	fn default() -> Self {
		Matrix4::IDENTITY
	}
}

impl Matrix4 {
	pub const ZERO: Self = Self {
		x_column: Vector4::new(0.0, 0.0, 0.0, 0.0),
		y_column: Vector4::new(0.0, 0.0, 0.0, 0.0),
		z_column: Vector4::new(0.0, 0.0, 0.0, 0.0),
		w_column: Vector4::new(0.0, 0.0, 0.0, 0.0),
	};

	pub const IDENTITY: Self = Self {
		x_column: Vector4::new(1.0, 0.0, 0.0, 0.0),
		y_column: Vector4::new(0.0, 1.0, 0.0, 0.0),
		z_column: Vector4::new(0.0, 0.0, 1.0, 0.0),
		w_column: Vector4::new(0.0, 0.0, 0.0, 1.0),
	};

	pub const fn from_cols(
		x_axis: Vector4,
		y_axis: Vector4,
		z_axis: Vector4,
		w_axis: Vector4,
	) -> Self {
		Self {
			x_column: x_axis,
			y_column: y_axis,
			z_column: z_axis,
			w_column: w_axis,
		}
	}

	pub const fn from_rows(
		x_axis: Vector4,
		y_axis: Vector4,
		z_axis: Vector4,
		w_axis: Vector4,
	) -> Self {
		let x = Vector4::new(x_axis.x, y_axis.x, z_axis.x, w_axis.x);
		let y = Vector4::new(x_axis.y, y_axis.y, z_axis.y, w_axis.y);
		let z = Vector4::new(x_axis.z, y_axis.z, z_axis.z, w_axis.z);
		let w = Vector4::new(x_axis.w, y_axis.w, z_axis.w, w_axis.w);
		Self::from_cols(x, y, z, w)
	}

	pub fn col(&self, index: usize) -> Vector4 {
		match index {
			0 => self.x_column,
			1 => self.y_column,
			2 => self.z_column,
			3 => self.w_column,
			_ => Vector4::ZERO,
		}
	}

	pub fn row(&self, index: usize) -> Vector4 {
		match index {
			0 => Vector4::new(
				self.x_column.x,
				self.y_column.x,
				self.z_column.x,
				self.w_column.x,
			),
			1 => Vector4::new(
				self.x_column.y,
				self.y_column.y,
				self.z_column.y,
				self.w_column.z,
			),
			2 => Vector4::new(
				self.x_column.z,
				self.y_column.z,
				self.z_column.z,
				self.w_column.z,
			),
			3 => Vector4::new(
				self.x_column.w,
				self.y_column.w,
				self.z_column.w,
				self.w_column.w,
			),
			_ => Vector4::ZERO,
		}
	}

	pub const fn ortho(width: f32, height: f32, far: f32, near: f32) -> Self {
		// NOTE: 0 - 1 z clipping
		let mut result = Matrix4::IDENTITY;
		result.x_column.x = 2.0 / width;
		result.y_column.y = 2.0 / height;
		result.z_column.z = 1.0 / (far - near);

		result.w_column.x = 0.0;
		result.w_column.y = 0.0;
		result.w_column.z = near / (far - near);
		result
	}

	pub fn perspective(fov: f32, aspect_ratio: f32, far: f32, near: f32) -> Self {
		let cotangent = 1.0 / f32::tan(fov * (PI / 360.0));

		let mut result = Matrix4::IDENTITY;
		result.x_column.x = cotangent / aspect_ratio;
		result.y_column.y = cotangent;
		result.z_column.w = -1.0;

		result.z_column.z = far / (near - far);
		result.w_column.z = -(far * near) / (far - near);

		result.w_column.w = 0.0;

		result
	}

	pub fn translate(xyz: impl Into<Vector3>) -> Matrix4 {
		let mut result = Matrix4::IDENTITY;
		result.w_column = (xyz.into(), 1.0).into();
		result
	}

	pub fn rotate(quat: impl Into<Quaternion>) -> Matrix4 {
		let normalized = quat.into().norm();

		let mut result = Matrix4::IDENTITY;

		let xx = normalized.x * normalized.x;
		let xy = normalized.x * normalized.y;
		let xz = normalized.x * normalized.z;
		let xw = normalized.x * normalized.w;

		let yy = normalized.y * normalized.y;
		let yz = normalized.y * normalized.z;

		let yw = normalized.y * normalized.w;
		let zz = normalized.z * normalized.z;
		let zw = normalized.z * normalized.w;

		result.x_column.x = 1.0 - 2.0 * (yy + zz);
		result.x_column.y = 2.0 * (xy + zw);
		result.x_column.z = 2.0 * (xz - yw);

		result.y_column.x = 2.0 * (xy - zw);
		result.y_column.y = 1.0 - 2.0 * (xx + zz);
		result.y_column.z = 2.0 * (yz + xw);

		result.z_column.x = 2.0 * (xz + yw);
		result.z_column.y = 2.0 * (yz - xw);
		result.z_column.z = 1.0 - 2.0 * (xx + yy);

		result
	}

	pub fn scale(xyz: impl Into<Vector3>) -> Self {
		let xyz = xyz.into();
		Self {
			x_column: Vector4::new(xyz.x, 0.0, 0.0, 0.0),
			y_column: Vector4::new(0.0, xyz.y, 0.0, 0.0),
			z_column: Vector4::new(0.0, 0.0, xyz.z, 0.0),
			w_column: Vector4::new(0.0, 0.0, 0.0, 1.0),
		}
	}

	pub fn inverse(self) -> Option<Self> {
		let mut inv = [0.0; 16];
		inv[0] = self[5] * self[10] * self[15]
			- self[5] * self[11] * self[14]
			- self[9] * self[6] * self[15]
			+ self[9] * self[7] * self[14]
			+ self[13] * self[6] * self[11]
			- self[13] * self[7] * self[10];

		inv[4] = -self[4] * self[10] * self[15]
			+ self[4] * self[11] * self[14]
			+ self[8] * self[6] * self[15]
			- self[8] * self[7] * self[14]
			- self[12] * self[6] * self[11]
			+ self[12] * self[7] * self[10];

		inv[8] = self[4] * self[9] * self[15]
			- self[4] * self[11] * self[13]
			- self[8] * self[5] * self[15]
			+ self[8] * self[7] * self[13]
			+ self[12] * self[5] * self[11]
			- self[12] * self[7] * self[9];

		inv[12] = -self[4] * self[9] * self[14]
			+ self[4] * self[10] * self[13]
			+ self[8] * self[5] * self[14]
			- self[8] * self[6] * self[13]
			- self[12] * self[5] * self[10]
			+ self[12] * self[6] * self[9];

		inv[1] = -self[1] * self[10] * self[15]
			+ self[1] * self[11] * self[14]
			+ self[9] * self[2] * self[15]
			- self[9] * self[3] * self[14]
			- self[13] * self[2] * self[11]
			+ self[13] * self[3] * self[10];

		inv[5] = self[0] * self[10] * self[15]
			- self[0] * self[11] * self[14]
			- self[8] * self[2] * self[15]
			+ self[8] * self[3] * self[14]
			+ self[12] * self[2] * self[11]
			- self[12] * self[3] * self[10];

		inv[9] = -self[0] * self[9] * self[15]
			+ self[0] * self[11] * self[13]
			+ self[8] * self[1] * self[15]
			- self[8] * self[3] * self[13]
			- self[12] * self[1] * self[11]
			+ self[12] * self[3] * self[9];

		inv[13] = self[0] * self[9] * self[14]
			- self[0] * self[10] * self[13]
			- self[8] * self[1] * self[14]
			+ self[8] * self[2] * self[13]
			+ self[12] * self[1] * self[10]
			- self[12] * self[2] * self[9];

		inv[2] = self[1] * self[6] * self[15]
			- self[1] * self[7] * self[14]
			- self[5] * self[2] * self[15]
			+ self[5] * self[3] * self[14]
			+ self[13] * self[2] * self[7]
			- self[13] * self[3] * self[6];

		inv[6] = -self[0] * self[6] * self[15]
			+ self[0] * self[7] * self[14]
			+ self[4] * self[2] * self[15]
			- self[4] * self[3] * self[14]
			- self[12] * self[2] * self[7]
			+ self[12] * self[3] * self[6];

		inv[10] = self[0] * self[5] * self[15]
			- self[0] * self[7] * self[13]
			- self[4] * self[1] * self[15]
			+ self[4] * self[3] * self[13]
			+ self[12] * self[1] * self[7]
			- self[12] * self[3] * self[5];

		inv[14] = -self[0] * self[5] * self[14]
			+ self[0] * self[6] * self[13]
			+ self[4] * self[1] * self[14]
			- self[4] * self[2] * self[13]
			- self[12] * self[1] * self[6]
			+ self[12] * self[2] * self[5];

		inv[3] = -self[1] * self[6] * self[11]
			+ self[1] * self[7] * self[10]
			+ self[5] * self[2] * self[11]
			- self[5] * self[3] * self[10]
			- self[9] * self[2] * self[7]
			+ self[9] * self[3] * self[6];

		inv[7] = self[0] * self[6] * self[11]
			- self[0] * self[7] * self[10]
			- self[4] * self[2] * self[11]
			+ self[4] * self[3] * self[10]
			+ self[8] * self[2] * self[7]
			- self[8] * self[3] * self[6];

		inv[11] = -self[0] * self[5] * self[11]
			+ self[0] * self[7] * self[9]
			+ self[4] * self[1] * self[11]
			- self[4] * self[3] * self[9]
			- self[8] * self[1] * self[7]
			+ self[8] * self[3] * self[5];

		inv[15] = self[0] * self[5] * self[10]
			- self[0] * self[6] * self[9]
			- self[4] * self[1] * self[10]
			+ self[4] * self[2] * self[9]
			+ self[8] * self[1] * self[6]
			- self[8] * self[2] * self[5];

		let det = self[0] * inv[0] + self[1] * inv[4] + self[2] * inv[8] + self[3] * inv[12];
		if det == 0.0 {
			return None;
		}
		let det = 1.0 / det;

		let mut result = Matrix4::ZERO;
		for i in 0..16 {
			result[i] = inv[i] * det;
		}
		Some(result)
	}
}

impl Index<usize> for Matrix4 {
	type Output = f32;

	fn index(&self, index: usize) -> &Self::Output {
		let col = index / 4;
		let row = index % 4;

		let col = match col {
			0 => (&self.x_column),
			1 => (&self.y_column),
			2 => (&self.z_column),
			3 => (&self.w_column),
			_ => unreachable!(),
		};

		&col[row]
	}
}

impl IndexMut<usize> for Matrix4 {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		let col = index / 4;
		let row = index % 4;

		let col = match col {
			0 => (&mut self.x_column),
			1 => (&mut self.y_column),
			2 => (&mut self.z_column),
			3 => (&mut self.w_column),
			_ => unreachable!(),
		};

		&mut col[row]
	}
}

impl Mul for Matrix4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut row_x = Vector4::ZERO;
		row_x.x = self.row(0).dot(rhs.x_column);
		row_x.y = self.row(0).dot(rhs.y_column);
		row_x.z = self.row(0).dot(rhs.z_column);
		row_x.w = self.row(0).dot(rhs.w_column);

		let mut row_y = Vector4::ZERO;
		row_y.x = self.row(1).dot(rhs.x_column);
		row_y.y = self.row(1).dot(rhs.y_column);
		row_y.z = self.row(1).dot(rhs.z_column);
		row_y.w = self.row(1).dot(rhs.w_column);

		let mut row_z = Vector4::ZERO;
		row_z.x = self.row(2).dot(rhs.x_column);
		row_z.y = self.row(2).dot(rhs.y_column);
		row_z.z = self.row(2).dot(rhs.z_column);
		row_z.w = self.row(2).dot(rhs.w_column);

		let mut row_w = Vector4::ZERO;
		row_w.x = self.row(3).dot(rhs.x_column);
		row_w.y = self.row(3).dot(rhs.y_column);
		row_w.z = self.row(3).dot(rhs.z_column);
		row_w.w = self.row(3).dot(rhs.w_column);

		Self::from_rows(row_x, row_y, row_z, row_w)
	}
}

impl Mul<Vector4> for Matrix4 {
	type Output = Vector4;

	fn mul(self, rhs: Vector4) -> Self::Output {
		let x = self.row(0).dot(rhs);
		let y = self.row(1).dot(rhs);
		let z = self.row(2).dot(rhs);
		let w = self.row(3).dot(rhs);

		Vector4::new(x, y, z, w)
	}
}

impl MulAssign for Matrix4 {
	fn mul_assign(&mut self, rhs: Self) {
		*self = *self * rhs;
	}
}
