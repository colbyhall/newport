#![feature(const_fn_floating_point_arithmetic)]
#![feature(const_float_classify)]
#![feature(const_trait_impl)]
#![feature(trait_alias)]
#![allow(clippy::float_cmp)]

#[allow(clippy::approx_constant)]
pub const PI: f32 = 3.141592;
pub const TAU: f32 = PI * 2.0;

pub const TO_RAD: f32 = PI / 180.0;
pub const TO_DEG: f32 = 180.0 / PI;

pub const SMALL_NUMBER: f32 = 1.0e-8;

mod color;
mod mat3;
mod mat4;
mod quat;
mod rect;
mod vec2;
mod vec3;
mod vec4;

pub use {
	color::*,
	mat3::*,
	mat4::*,
	quat::*,
	rect::*,
	vec2::*,
	vec3::*,
	vec4::*,
};

use serde::{
	de::DeserializeOwned,
	Serialize,
};

use std::ops::*;

pub trait Float:
	Default
	+ Add<Output = Self>
	+ AddAssign
	+ Sub<Output = Self>
	+ SubAssign
	+ Mul<Output = Self>
	+ MulAssign
	+ Div<Output = Self>
	+ DivAssign
	+ Copy
	+ Clone
	+ PartialEq
	+ DeserializeOwned
	+ Serialize
{
	const ZERO: Self;
	const ONE: Self;
}

macro_rules! add_impl_float {
    ($($t:ty)*) => ($(
        impl Float for $t {
			const ZERO: Self = 0.0;
			const ONE: Self = 1.0;
        }
    )*)
}

add_impl_float! { f32 f64 }

pub fn lerp<T: Float>(a: T, b: T, t: T) -> T {
	(T::ONE - t) * a + t * b
}

#[cfg(test)]
mod test {
	#[test]
	fn lerp() {
		let x: f32 = super::lerp(0.0, 1.0, 0.5);
		assert_eq!(x, 0.5);

		let x: f32 = super::lerp(0.0, 100.0, 0.5);
		assert_eq!(x, 50.0);

		let x: f64 = super::lerp(0.0, 1.0, 0.5);
		assert_eq!(x, 0.5);

		let x: f64 = super::lerp(0.0, 100.0, 0.5);
		assert_eq!(x, 50.0);
	}
}
