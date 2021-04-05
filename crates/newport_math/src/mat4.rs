use core::ops::{ Mul, MulAssign, };

use crate::Vector4;
use crate::Vector3;

#[allow(unused_imports)]
use num_traits::*;

#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
pub struct Matrix4 {
    pub x_axis: Vector4,
    pub y_axis: Vector4,
    pub z_axis: Vector4,
    pub w_axis: Vector4,
}

impl Matrix4 {
    pub const ZERO: Self = Self{
        x_axis: Vector4::new(0.0, 0.0, 0.0, 0.0),
        y_axis: Vector4::new(0.0, 0.0, 0.0, 0.0),
        z_axis: Vector4::new(0.0, 0.0, 0.0, 0.0),
        w_axis: Vector4::new(0.0, 0.0, 0.0, 0.0),
    };

    pub const IDENTITY: Self = Self{
        x_axis: Vector4::new(1.0, 0.0, 0.0, 0.0),
        y_axis: Vector4::new(0.0, 1.0, 0.0, 0.0),
        z_axis: Vector4::new(0.0, 0.0, 1.0, 0.0),
        w_axis: Vector4::new(0.0, 0.0, 0.0, 1.0),
    };

    pub const fn from_cols(x_axis: Vector4, y_axis: Vector4, z_axis: Vector4, w_axis: Vector4) -> Self {
        Self{ x_axis: x_axis, y_axis: y_axis, z_axis: z_axis, w_axis: w_axis }
    }

    pub const fn from_rows(x_axis: Vector4, y_axis: Vector4, z_axis: Vector4, w_axis: Vector4) -> Self {
        let x = Vector4::new(x_axis.x, y_axis.x, z_axis.x, w_axis.x);
        let y = Vector4::new(x_axis.y, y_axis.y, z_axis.y, w_axis.y);
        let z = Vector4::new(x_axis.z, y_axis.z, z_axis.z, w_axis.z);
        let w = Vector4::new(x_axis.w, y_axis.w, z_axis.w, w_axis.w);
        Self::from_cols(x, y, z, w)
    }

    pub fn col(&self, index: usize) -> Vector4 {
        match index {
            0 => self.x_axis,
            1 => self.y_axis,
            2 => self.z_axis,
            3 => self.w_axis,
            _ => Vector4::ZERO,
        }
    }

    pub fn row(&self, index: usize) -> Vector4 {
        match index {
            0 => Vector4::new(self.x_axis.x, self.y_axis.x, self.z_axis.x, self.w_axis.x),
            1 => Vector4::new(self.x_axis.y, self.y_axis.y, self.z_axis.y, self.w_axis.z),
            2 => Vector4::new(self.x_axis.z, self.y_axis.z, self.z_axis.z, self.w_axis.z),
            3 => Vector4::new(self.x_axis.w, self.y_axis.w, self.z_axis.w, self.w_axis.w),
            _ => Vector4::ZERO,
        }
    }

    pub const fn ortho(width: f32, height: f32, far: f32, near: f32) -> Self {
        // NOTE: 0 - 1 z clipping
        let mut result = Matrix4::IDENTITY;
        result.x_axis.x = 2.0 / width;
        result.y_axis.y = 2.0 / height;
        result.z_axis.z = 1.0 / (far - near);

        result.w_axis.x = 0.0;
        result.w_axis.y = 0.0;
        result.w_axis.z = near / (far - near);
        result
    }

    pub fn translate(xyz: Vector3) -> Matrix4 {
        let mut result = Matrix4::IDENTITY;
        result.w_axis = (xyz, 1.0).into();
        result
    }
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut row_x = Vector4::ZERO;
        row_x.x = self.row(0).dot(rhs.x_axis);
        row_x.y = self.row(0).dot(rhs.y_axis);
        row_x.z = self.row(0).dot(rhs.z_axis);
        row_x.w = self.row(0).dot(rhs.w_axis);

        let mut row_y = Vector4::ZERO;
        row_y.x = self.row(1).dot(rhs.x_axis);
        row_y.y = self.row(1).dot(rhs.y_axis);
        row_y.z = self.row(1).dot(rhs.z_axis);
        row_y.w = self.row(1).dot(rhs.w_axis);

        let mut row_z = Vector4::ZERO;
        row_z.x = self.row(2).dot(rhs.x_axis);
        row_z.y = self.row(2).dot(rhs.y_axis);
        row_z.z = self.row(2).dot(rhs.z_axis);
        row_z.w = self.row(2).dot(rhs.w_axis);

        let mut row_w = Vector4::ZERO;
        row_w.x = self.row(3).dot(rhs.x_axis);
        row_w.y = self.row(3).dot(rhs.y_axis);
        row_w.z = self.row(3).dot(rhs.z_axis);
        row_w.w = self.row(3).dot(rhs.w_axis);
        
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
