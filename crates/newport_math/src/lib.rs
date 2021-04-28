#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_float_classify)]

pub const PI  : f32 = 3.141592;
pub const TAU : f32 = PI * 2.0;

pub const TO_RAD : f32 = PI / 180.0;
pub const TO_DEG : f32 = 180.0 / PI;

pub mod vec2;
pub use vec2::*;

pub mod vec3;
pub use vec3::*;

pub mod vec4;
pub use vec4::*;

pub mod mat4;
pub use mat4::*;

pub mod color;
pub use color::*;

pub mod rect;
pub use rect::*;

pub mod quat;
pub use quat::*;

pub fn min(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}

pub fn max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}