#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_float_classify)]

#[allow(clippy::approx_constant)]
pub const PI: f32 = 3.141592;
pub const TAU: f32 = PI * 2.0;

pub const TO_RAD: f32 = PI / 180.0;
pub const TO_DEG: f32 = 180.0 / PI;

pub const SMALL_NUMBER: f32 = 1.0e-8;

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

pub trait InterpTo {
	fn interp_to(self, target: Self, dt: f32, speed: f32) -> Self;
}

impl InterpTo for f32 {
	fn interp_to(self, target: Self, dt: f32, speed: f32) -> Self {
		if speed <= 0.0 {
			return target;
		}

		let distance = target - self;
		if distance * distance < SMALL_NUMBER {
			return target;
		}

		let delta = distance * (dt * speed).max(0.0).min(1.0);
		self + delta
	}
}
