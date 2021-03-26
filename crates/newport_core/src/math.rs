use std::ops::*;
use serde::{ Serialize, Deserialize };

pub const PI        : f32 = 3.1415;
pub const TAU       : f32 = PI * 2.0;
pub const TO_RAD    : f32 = PI / 180.0;
pub const TO_DEG    : f32 = 180.0 / PI;

pub const V2_UP     : Vector2 = Vector2 { x: 0.0, y: 1.0 };
pub const V2_RIGHT  : Vector2 = Vector2 { x: 1.0, y: 0.0 };

pub const SMALL_NUMBER : f32 = 1.0e-8;

#[macro_export]
macro_rules! min {
    ($a:expr, $b:expr) => ( if $a < $b { $a } else { $b } )
}

#[macro_export]
macro_rules! max {
    ($a:expr, $b:expr) => ( if $a > $b { $a } else { $b } )
}

#[macro_export]
macro_rules! clamp {
    ($x:expr, $min:expr, $max:expr) => ( min!(max!($x, $min), $max) )
}

#[inline]
pub fn interp_to(current: f32, target: f32, dt: f32, speed: f32) -> f32 {
    if speed <= 0.0 { return target; }

    let distance = target - current;
    if distance * distance < SMALL_NUMBER {
        return target;
    }

    let delta_move = distance * clamp!(dt * speed, 0.0, 1.0);
    current + delta_move
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[inline]
pub fn v2z() -> Vector2 { Vector2::default() }
#[inline]
pub fn v2s(xy: f32) -> Vector2 { Vector2 { x: xy, y: xy } }
#[inline]
pub fn v2(x: f32, y: f32) -> Vector2 { Vector2 { x, y } }

impl Vector2 {
    pub fn len_sq(&self) -> f32 { self.x * self.x + self.y * self.y }

    pub fn len(&self) -> f32 { self.len_sq().sqrt() }

    pub fn norm(self) -> Self { 
        let len = self.len();
        if len > 0.0 {
            return self.clone() / len;
        }
        v2s(0.0)
    }

    pub fn dot(self, rhs: Self) -> f32 { self.x * rhs.x + self.y * rhs.y }

    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn perp(self) -> Self {
        v2(self.y, -self.x)
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<f32> for Vector2 {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl AddAssign<f32> for Vector2 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<f32> for Vector2 {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl SubAssign for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl SubAssign<f32> for Vector2 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl Mul for Vector2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Mul<f32> for Vector2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign for Vector2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div for Vector2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Div<f32> for Vector2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign for Vector2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /=  rhs.y;
    }
}

impl DivAssign<f32> for Vector2 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /=  rhs;
    }
}

impl Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn v3z() -> Vector3 { Vector3::default() }
pub fn v3s(xyz: f32) -> Vector3 { Vector3 { x: xyz, y: xyz, z: xyz } }
pub fn v3(x: f32, y: f32, z: f32) -> Vector3 { Vector3 { x, y, z } }
pub fn v3xy(xy: Vector2, z: f32) -> Vector3 { Vector3 { x: xy.x, y: xy.y, z } }
pub fn v3yz(x: f32, yz: Vector2) -> Vector3 { Vector3 { x, y: yz.y, z: yz.y } }

impl Vector3 {
    pub fn len_sq(&self) -> f32 { self.x * self.x + self.y * self.y + self.z * self.z }

    pub fn len(&self) -> f32 { self.len_sq().sqrt() }

    pub fn norm(self) -> Self { 
        let len = self.len();
        if len > 0.0 {
            return self.clone() / len;
        }
        v3s(0.0)
    }

    pub fn dot(self, rhs: Self) -> f32 { self.x * rhs.x + self.y * rhs.y + self.z * rhs.z }

    pub fn cross(self, rhs: Self) -> Self {
        let x = self.y * rhs.z - self.z * rhs.y;
        let y = self.z * rhs.x - self.x * rhs.z;
        let z = self.x * rhs.y - self.y * rhs.x;
        Self { x, y, z }
    }

    pub fn xy(self) -> (Vector2, f32) {
        (v2(self.x, self.y), self.z)
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<f32> for Vector3 {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl AddAssign<f32> for Vector3 {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<f32> for Vector3 {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl SubAssign<f32> for Vector3 {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}

impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign for Vector3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div for Vector3 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign for Vector3 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

pub fn v4z() -> Vector4 { Vector4::default() }
pub fn v4(x: f32, y: f32, z: f32, w: f32) -> Vector4 { Vector4 { x, y, z, w } }
pub fn v4s(xyzw: f32) -> Vector4 { Vector4 { x: xyzw, y: xyzw, z: xyzw, w: xyzw } }

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Matrix4 {
    pub e: [f32; 4 * 4],
}

impl Matrix4 {
    pub fn identity() -> Self {
        let mut result = Self::default();
        for i in 0..4 {
            result.e[i + i * 4] = 1.0;
        }
        result
    }

    pub fn ortho(size: f32, aspect_ratio: f32, far: f32, near: f32) -> Matrix4 {
        let right = size * aspect_ratio;
        let left = -right;

        let top = size;
        let bottom = -size;

        let mut result = Matrix4::identity();
        result.e[0 + 0 * 4] =  2.0 / (right - left);
        result.e[1 + 1 * 4] =  2.0 / (top - bottom);
        result.e[2 + 2 * 4] = -2.0 / (far - near);
    
        result.e[0 + 3 * 4] = -((right + left)  / (right - left));
        result.e[1 + 3 * 4] = -((top + bottom)  / (top - bottom));
        result.e[2 + 3 * 4] = -((far + near)    / (far - near));
        result
    }

    pub fn translate(t: Vector3) -> Matrix4 {
        let mut result = Matrix4::identity();
        result.e[0 + 3 * 4] = t.x;
        result.e[1 + 3 * 4] = t.y;
        result.e[2 + 3 * 4] = t.z;
        result
    }

    pub fn inverse(self) -> Self {
        let mut result = self;

        let mut temp = [0.0; 16]; 
    
        temp[0] = result.e[5] * result.e[10] * result.e[15] -
            result.e[5] * result.e[11] * result.e[14] -
            result.e[9] * result.e[6] * result.e[15] +
            result.e[9] * result.e[7] * result.e[14] +
            result.e[13] * result.e[6] * result.e[11] -
            result.e[13] * result.e[7] * result.e[10];
    
        temp[4] = -result.e[4] * result.e[10] * result.e[15] +
            result.e[4] * result.e[11] * result.e[14] +
            result.e[8] * result.e[6] * result.e[15] -
            result.e[8] * result.e[7] * result.e[14] -
            result.e[12] * result.e[6] * result.e[11] +
            result.e[12] * result.e[7] * result.e[10];
    
        temp[8] = result.e[4] * result.e[9] * result.e[15] -
            result.e[4] * result.e[11] * result.e[13] -
            result.e[8] * result.e[5] * result.e[15] +
            result.e[8] * result.e[7] * result.e[13] +
            result.e[12] * result.e[5] * result.e[11] -
            result.e[12] * result.e[7] * result.e[9];
    
        temp[12] = -result.e[4] * result.e[9] * result.e[14] +
            result.e[4] * result.e[10] * result.e[13] +
            result.e[8] * result.e[5] * result.e[14] -
            result.e[8] * result.e[6] * result.e[13] -
            result.e[12] * result.e[5] * result.e[10] +
            result.e[12] * result.e[6] * result.e[9];
    
        temp[1] = -result.e[1] * result.e[10] * result.e[15] +
            result.e[1] * result.e[11] * result.e[14] +
            result.e[9] * result.e[2] * result.e[15] -
            result.e[9] * result.e[3] * result.e[14] -
            result.e[13] * result.e[2] * result.e[11] +
            result.e[13] * result.e[3] * result.e[10];
    
        temp[5] = result.e[0] * result.e[10] * result.e[15] -
            result.e[0] * result.e[11] * result.e[14] -
            result.e[8] * result.e[2] * result.e[15] +
            result.e[8] * result.e[3] * result.e[14] +
            result.e[12] * result.e[2] * result.e[11] -
            result.e[12] * result.e[3] * result.e[10];
    
        temp[9] = -result.e[0] * result.e[9] * result.e[15] +
            result.e[0] * result.e[11] * result.e[13] +
            result.e[8] * result.e[1] * result.e[15] -
            result.e[8] * result.e[3] * result.e[13] -
            result.e[12] * result.e[1] * result.e[11] +
            result.e[12] * result.e[3] * result.e[9];
    
        temp[13] = result.e[0] * result.e[9] * result.e[14] -
            result.e[0] * result.e[10] * result.e[13] -
            result.e[8] * result.e[1] * result.e[14] +
            result.e[8] * result.e[2] * result.e[13] +
            result.e[12] * result.e[1] * result.e[10] -
            result.e[12] * result.e[2] * result.e[9];
    
        temp[2] = result.e[1] * result.e[6] * result.e[15] -
            result.e[1] * result.e[7] * result.e[14] -
            result.e[5] * result.e[2] * result.e[15] +
            result.e[5] * result.e[3] * result.e[14] +
            result.e[13] * result.e[2] * result.e[7] -
            result.e[13] * result.e[3] * result.e[6];
    
        temp[6] = -result.e[0] * result.e[6] * result.e[15] +
            result.e[0] * result.e[7] * result.e[14] +
            result.e[4] * result.e[2] * result.e[15] -
            result.e[4] * result.e[3] * result.e[14] -
            result.e[12] * result.e[2] * result.e[7] +
            result.e[12] * result.e[3] * result.e[6];
    
        temp[10] = result.e[0] * result.e[5] * result.e[15] -
            result.e[0] * result.e[7] * result.e[13] -
            result.e[4] * result.e[1] * result.e[15] +
            result.e[4] * result.e[3] * result.e[13] +
            result.e[12] * result.e[1] * result.e[7] -
            result.e[12] * result.e[3] * result.e[5];
    
        temp[14] = -result.e[0] * result.e[5] * result.e[14] +
            result.e[0] * result.e[6] * result.e[13] +
            result.e[4] * result.e[1] * result.e[14] -
            result.e[4] * result.e[2] * result.e[13] -
            result.e[12] * result.e[1] * result.e[6] +
            result.e[12] * result.e[2] * result.e[5];
    
        temp[3] = -result.e[1] * result.e[6] * result.e[11] +
            result.e[1] * result.e[7] * result.e[10] +
            result.e[5] * result.e[2] * result.e[11] -
            result.e[5] * result.e[3] * result.e[10] -
            result.e[9] * result.e[2] * result.e[7] +
            result.e[9] * result.e[3] * result.e[6];
    
        temp[7] = result.e[0] * result.e[6] * result.e[11] -
            result.e[0] * result.e[7] * result.e[10] -
            result.e[4] * result.e[2] * result.e[11] +
            result.e[4] * result.e[3] * result.e[10] +
            result.e[8] * result.e[2] * result.e[7] -
            result.e[8] * result.e[3] * result.e[6];
    
        temp[11] = -result.e[0] * result.e[5] * result.e[11] +
            result.e[0] * result.e[7] * result.e[9] +
            result.e[4] * result.e[1] * result.e[11] -
            result.e[4] * result.e[3] * result.e[9] -
            result.e[8] * result.e[1] * result.e[7] +
            result.e[8] * result.e[3] * result.e[5];
    
        temp[15] = result.e[0] * result.e[5] * result.e[10] -
            result.e[0] * result.e[6] * result.e[9] -
            result.e[4] * result.e[1] * result.e[10] +
            result.e[4] * result.e[2] * result.e[9] +
            result.e[8] * result.e[1] * result.e[6] -
            result.e[8] * result.e[2] * result.e[5];
    
        let mut determinant = result.e[0] * temp[0] + result.e[1] * temp[4] + result.e[2] * temp[8] + result.e[3] * temp[12];
        determinant = 1.0 / determinant;
    
        for i in 0..16 {
            result.e[i] = temp[i] * determinant;
        }
    
        return result;
    }
}

impl Mul<Matrix4> for Matrix4 {
    type Output = Self;
    fn mul(self, rhs: Matrix4) -> Self::Output {
        let mut result = Matrix4::default();
        for y in 0..4 {
            for x in 0..4 {
                let mut sum = 0.0;
                for e in 0..4 {
                    sum += self.e[x + e * 4] * rhs.e[e + y * 4];
                }
                result.e[x + y * 4] = sum;
            }
        }
        result
    }
}

impl MulAssign<Matrix4> for Matrix4 {
    fn mul_assign(&mut self, rhs: Matrix4) {
        *self = *self * rhs;
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Self::Output {
        Self::Output {
            x: self.e[0 + 0 * 4] * rhs.x + self.e[1 + 0 * 4] * rhs.y + self.e[2 + 0 * 4] * rhs.z + self.e[3 + 0 * 4] * rhs.z,
            y: self.e[0 + 1 * 4] * rhs.x + self.e[1 + 1 * 4] * rhs.y + self.e[2 + 1 * 4] * rhs.z + self.e[3 + 1 * 4] * rhs.z,
            z: self.e[0 + 2 * 4] * rhs.x + self.e[1 + 2 * 4] * rhs.y + self.e[2 + 2 * 4] * rhs.z + self.e[3 + 2 * 4] * rhs.z,
            w: self.e[0 + 3 * 4] * rhs.x + self.e[1 + 3 * 4] * rhs.y + self.e[2 + 3 * 4] * rhs.z + self.e[3 + 3 * 4] * rhs.z,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32, 
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn v3(self) -> (Vector3, f32) {
        (v3(self.r, self.g, self.b), self.a)
    }

    pub fn v4(self) -> Vector4 {
        v4(self.r, self.g, self.b, self.a)
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output { r: self.r * rhs, g: self.g * rhs, b: self.g * rhs, a: self.a * rhs }
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl From<u32> for Color {
    fn from(item: u32) -> Self {
        let r = (item & 0xFF000000) >> 24;
        let g = (item & 0x00FF0000) >> 16;
        let b = (item & 0x0000FF00) >> 8;
        let a =  item & 0x000000FF;

        Self {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }
}

pub mod colors {
    use super::Color;

    pub const RED     : Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN   : Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE    : Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const WHITE   : Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK   : Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const CYAN    : Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const YELLOW  : Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const MAGENTA : Color = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
}

pub fn line_intersect_2d(a1: Vector2, a2: Vector2, b1: Vector2, b2: Vector2) -> Option<Vector2> {
    let a = a2 - a1;
    let b = b2 - b1;

    let ab_cross = a.cross(b);
    if ab_cross == 0.0 { return None; }

    let c = b1 - a1;
    let t = c.cross(b) / ab_cross;
    if t < 0.0 || t > 1.0 { return None; }

    let u = c.cross(a) / ab_cross;
    if u < 0.0 || u > 1.0 { return None; }

    Some(a1 + a * t)
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vector2,
    pub max: Vector2,
}

impl Rect {
    pub fn from_raw(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { min: v2(x0, y0), max: v2(x1, y1) }
    }

    pub fn from_min_max(min: Vector2, max: Vector2) -> Self {
        Self { min, max }
    }

    pub fn from_pos_size(pos: Vector2, size: Vector2) -> Self {
        let min = pos - (size / 2.0);
        let max = pos + (size / 2.0);
        Self::from_min_max(min, max)
    }

    pub fn overlaps_point(self, p: Vector2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn overlaps_rect(self, rhs: Self) -> Option<Self> {
        if rhs.min.x >= self.min.x && rhs.max.x <= self.max.x && rhs.min.y >= self.min.y && rhs.max.y <= self.max.y {
            let min = v2(min!(self.min.x, rhs.min.x), min!(self.min.y, rhs.min.y));
            let max = v2(min!(self.max.x, rhs.max.x), min!(self.max.y, rhs.max.y));
            return Some(Self::from_min_max(min, max));
        }
        None
    }

    pub fn intersect_line(self, b1: Vector2, b2: Vector2) -> Option<(Vector2, Vector2)> {
        let dir = (b2 - b1).norm();
        let dot_up = V2_UP.dot(dir);
        let dot_right = V2_RIGHT.dot(dir);

        if dot_right > 0.5 {
            let a1 = self.min;
            let a2 = v2(self.min.x, self.max.y);
            let i = line_intersect_2d(a1, a2, b1, b2);
            if i.is_some() {
                let i = i.unwrap();
                let a_dir = (a2 - a1).norm();
                return Some((i, a_dir.perp()));
            }
        }

        if dot_right < 0.5 {
            let a1 = self.max;
            let a2 = v2(self.max.x, self.min.y);
            let i = line_intersect_2d(a1, a2, b1, b2);
            if i.is_some() {
                let i = i.unwrap();
                let a_dir = (a2 - a1).norm();
                return Some((i, a_dir.perp()));
            }
        }

        if dot_up > 0.5 {
            let a1 = self.min;
            let a2 = v2(self.max.x, self.min.y);
            let i = line_intersect_2d(a1, a2, b1, b2);
            if i.is_some() {
                let i = i.unwrap();
                let a_dir = (a2 - a1).norm();
                return Some((i, a_dir.perp()));
            }
        }

        if dot_up < 0.5 {
            let a1 = self.max;
            let a2 = v2(self.min.x, self.max.y);
            let i = line_intersect_2d(a1, a2, b1, b2);
            if i.is_some() {
                let i = i.unwrap();
                let a_dir = (a2 - a1).norm();
                return Some((i, a_dir.perp()));
            }
        }

        None
    }

    pub fn sweep_rect(a1: Vector2, a2: Vector2, a_size: Vector2, b: Rect) -> Option<(Vector2, Vector2)> {
        let b_pos   = b.pos();
        let large_b = Rect::from_pos_size(b_pos, b.size() + a_size);

        let intersec = large_b.intersect_line(a1, a2);
        if intersec.is_some() {
            return intersec;
        }

        let at_end   = Rect::from_pos_size(a2, a_size);
        let intersec = at_end.overlaps_rect(b);
        if intersec.is_some() {
            let intersec       = intersec.unwrap();
            let intersec_size  = intersec.size();

            if intersec_size.x > intersec_size.y {
                let flip    = (a2.y - b_pos.y).signum();
                let impact  = a2 + v2(0.0, intersec_size.y * flip);
                let normal  = v2(0.0, flip);
                return Some((impact, normal));
            } else {
                let flip   = (a2.x - b_pos.x).signum();
                let impact = a2 + v2(intersec_size.x * flip, 0.0);
                let normal = v2(flip, 0.0);
                return Some((impact, normal));
            }
        }
        None
    }

    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(self) -> Vector2 {
        v2(self.width(), self.height())
    }

    pub fn pos(self) -> Vector2 {
        let x = self.min.x + self.width() / 2.0;
        let y = self.min.y + self.height() / 2.0;
        v2(x, y)
    }
}