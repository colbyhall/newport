use crate::{
    math,
};

use math::{
    Vector3,
    Quaternion,
    Matrix4
};

#[derive(Copy, Clone)]
pub struct Transform {
    pub location: Vector3,
    pub rotation: Quaternion,
    pub scale:    Vector3,
}

impl Transform {
    pub fn new(location: Vector3, rotation: Quaternion, scale: Vector3) -> Self {
        Self{ location, rotation, scale }
    }
}

impl From<Matrix4> for Transform {
    fn from(_m: Matrix4) -> Transform {
        todo!()
    }
}