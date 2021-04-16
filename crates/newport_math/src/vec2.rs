use std::ops::{ 
    Add, 
    AddAssign, 
    
    Sub, 
    SubAssign, 
    
    Mul, 
    MulAssign, 
    
    Div, 
    DivAssign, 
    
    Neg
};

use std::convert::From;

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Self  = Self::new(0.0, 0.0);

    pub const RIGHT: Self = Self::new(1.0, 0.0);
    pub const UP:    Self = Self::new(0.0, 1.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self{ x: x, y: y }
    }

    pub const fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub const fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    pub const fn perp(self) -> Self {
        Self::new(self.y, -self.x)
    }

    pub const fn len_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn len(self) -> f32 {
        self.len_sq().sqrt() }

    pub fn abs(self) -> Self {
        Vector2::new(self.x.abs(), self.y.abs())
    }

    pub const fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub const fn is_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    pub const fn min(self, other: Self) -> Self {
        let x = if self.x < other.x {
            self.x
        } else {
            other.x
        };

        let y = if self.y < other.y {
            self.y
        } else {
            other.y
        };

        Self::new(x, y)
    }

    pub const fn max(self, other: Self) -> Self {
        let x = if self.x > other.x {
            self.x
        } else {
            other.x
        };

        let y = if self.y > other.y {
            self.y
        } else {
            other.y
        };

        Self::new(x, y)
    }

    pub const fn min_elem(self) -> f32 {
        if self.x < self.y {
            self.x
        } else {
            self.y
        }
    }

    pub const fn max_elem(self) -> f32 {
        if self.x > self.y {
            self.x
        } else {
            self.y
        }
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

impl From<(f32, f32)> for Vector2 {
    fn from(xy: (f32, f32)) -> Self {
        let (x, y) = xy;
        Self{ x, y }
    }
}