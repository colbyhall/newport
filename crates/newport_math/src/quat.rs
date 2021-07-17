use serde::{Deserialize, Serialize};

use crate::Vector3;

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn from_axis_angle(axis: impl Into<Vector3>, theta: f32) -> Self {
        let axis = axis.into();

        let theta = theta / 2.0;

        let s = theta.sin();
        let c = theta.cos();
        Self {
            x: s * axis.x,
            y: s * axis.y,
            z: s * axis.z,
            w: c,
        }
    }

    pub fn from_euler(euler: impl Into<Vector3>) -> Self {
        let euler = euler.into() * 0.5;

        let cr = euler.x.cos();
        let sr = euler.x.sin();

        let cp = euler.y.cos();
        let sp = euler.y.sin();

        let cy = euler.z.cos();
        let sy = euler.z.sin();

        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }

    pub fn len_sq(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn len(self) -> f32 {
        self.len_sq().sqrt()
    }

    pub fn norm(self) -> Self {
        let len = self.len();

        // TODO: Use a threshold
        if len == 0.0 {
            Default::default()
        } else {
            let inv = 1.0 / len;
            Self {
                x: self.x * inv,
                y: self.y * inv,
                z: self.z * inv,
                w: self.w * inv,
            }
        }
    }

    pub fn inverse(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn rotate(self, xyz: Vector3) -> Vector3 {
        let t = self.xyz().cross(xyz) * 2.0 * self.w;
        (xyz + t) + self.xyz().cross(t)
    }

    pub fn forward(self) -> Vector3 {
        self.rotate(Vector3::FORWARD)
    }

    pub fn right(self) -> Vector3 {
        self.rotate(Vector3::RIGHT)
    }

    pub fn up(self) -> Vector3 {
        self.rotate(Vector3::UP)
    }

    pub fn xyz(self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
