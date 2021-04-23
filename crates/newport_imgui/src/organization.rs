use crate::math::{ Rect, Vector2 };

#[derive(Clone)]
pub enum Sizing {
    MinMax(Vector2, Vector2),
    Fill(bool, bool)
}

#[derive(Clone)]
pub struct Organization {
    pub margin:  Rect,
    pub padding: Rect,

    pub sizing:  Sizing,
}

impl Organization {
    pub fn fill(margin: impl Into<Rect>, width: bool, height: bool) -> Self {
        Self {
            margin: margin.into(),
            padding: Rect::default(),

            sizing: Sizing::Fill(width, height)
        }
    }
}

impl Organization {
    pub fn content_size(&self, mut needed: Vector2, available: Vector2) -> Vector2 {
        needed += self.padding.min + self.padding.max;

        match self.sizing {
            Sizing::MinMax(min, max) => {
                let needed = needed.max(min);
                needed.min(max)
            },
            Sizing::Fill(width, height) => {
                if width {
                    needed.x = available.x - self.margin.width();
                }

                if height {
                    needed.y = available.y - self.margin.height();
                }

                needed
            }
        }
    }

    pub fn spacing_size(&self, size: Vector2) -> Vector2 {
        size + self.margin.min + self.margin.max
    }
}

impl Default for Organization {
    fn default() -> Self {
        Self {
            margin:  (5.0, 5.0, 5.0, 5.0).into(),
            padding: (10.0, 10.0, 10.0, 10.0).into(),

            sizing: Sizing::MinMax(-Vector2::INFINITY, Vector2::INFINITY)
        }
    }
}