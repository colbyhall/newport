#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

#[cfg(not(target_arch = "spirv"))]
#[macro_use]
pub extern crate spirv_std_macros;
use newport_math::{ Color, Vector3, Vector4 };

#[spirv(fragment)]
pub fn main_fs(color: Color, output: &mut Vector4) {
    *output = color.into();
}

#[spirv(vertex)]
pub fn main_vs(
    position: Vector3,
    color:    Color,
    out_color: &mut Color,
    #[spirv(position, invariant)] out_pos: &mut Vector4,
) {
    *out_color = color.into();
    *out_pos = (position, 1.0).into();
}