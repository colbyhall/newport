use crate::Vector2;

#[derive(Copy, Clone)]
pub struct Rect {
    pub min: Vector2,
    pub max: Vector2,
}

impl Rect {
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