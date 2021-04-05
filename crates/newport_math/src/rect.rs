use crate::Vector2;

use core::convert::From;

#[allow(unused_imports)]
use num_traits::*;

#[derive(Copy, Clone, Default, Debug)]
pub struct Rect {
    pub min: Vector2,
    pub max: Vector2,
}

impl Rect {
    pub fn from_min_max(min: Vector2, max: Vector2) -> Self {
        Self{
            min: min,
            max: max,
        }
    }

    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(self) -> Vector2 {
        Vector2::new(self.width(), self.height())
    }

    pub fn pos(self) -> Vector2 {
        let x = self.min.x + self.width() / 2.0;
        let y = self.min.y + self.height() / 2.0;
        Vector2::new(x, y)
    }
}

impl From<(Vector2, Vector2)> for Rect {
    fn from(min_max: (Vector2, Vector2)) -> Self {
        let (min, max) = min_max;
        Self{
            min: min,
            max: max,
        }
    }
}