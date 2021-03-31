use core::ops::{ Mul, MulAssign, };

use crate::Vector4;

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
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
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut rows = [Vector4::ZERO; 4];
        for (index, it) in rows.iter_mut().enumerate() {
            it.x = self.row(index).dot(rhs.x_axis);
            it.y = self.row(index).dot(rhs.y_axis);
            it.z = self.row(index).dot(rhs.z_axis);
            it.w = self.row(index).dot(rhs.w_axis);
        }
        Self::from_rows(rows[0], rows[1], rows[2], rows[3])
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Vector4) -> Self::Output {
        let mut rows = [Vector4::ZERO; 4];
        for (index, it) in rows.iter_mut().enumerate() {
            it.x = self.row(index).dot(rhs.x.into());
            it.y = self.row(index).dot(rhs.y.into());
            it.z = self.row(index).dot(rhs.z.into());
            it.w = self.row(index).dot(rhs.w.into());
        }
        Self::from_rows(rows[0], rows[1], rows[2], rows[3])
    }
}

impl MulAssign for Matrix4 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<Vector4> for Matrix4 {
    fn mul_assign(&mut self, rhs: Vector4) {
        *self = *self * rhs;
    }
}
