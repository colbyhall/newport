use crate::Vector2;

use std::convert::From;

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

    pub fn bottom_left(self) -> Vector2 {
        self.min
    }

    pub fn top_right(self) -> Vector2 {
        self.max
    }

    pub fn bottom_right(self) -> Vector2 {
        (self.max.x, self.min.y).into()
    }

    pub fn top_left(self) -> Vector2 {
        (self.min.x, self.max.y).into()
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

impl From<(f32, f32, f32, f32)> for Rect {
    fn from(rect: (f32, f32, f32, f32)) -> Self {
        let (x0, y0, x1, y1) = rect;
        Self {
            min: Vector2::new(x0, y0),
            max: Vector2::new(x1, y1)
        }
    }
}