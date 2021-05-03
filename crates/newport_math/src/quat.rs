use serde::{ Serialize, Deserialize };

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}