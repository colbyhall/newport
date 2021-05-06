use crate::math::{ Rect, Vector2 };

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    UpToDown,
    DownToUp,
}

#[derive(Copy, Clone, Debug)]
pub struct Layout {
    bounds:     Rect,
    direction:  Direction,
    cursor:     f32,
}

impl Layout {
    pub fn up_to_down(bounds: impl Into<Rect>) -> Layout {
        Layout {
            bounds: bounds.into(),
            direction: Direction::UpToDown,
            cursor: 0.0
        }
    }
    
    pub fn down_to_up(bounds: impl Into<Rect>) -> Layout {
        Layout {
            bounds: bounds.into(),
            direction: Direction::DownToUp,
            cursor: 0.0
        }
    }

    pub fn left_to_right(bounds: impl Into<Rect>) -> Layout {
        Layout {
            bounds: bounds.into(),
            direction: Direction::LeftToRight,
            cursor: 0.0
        }
    }

    pub fn right_to_left(bounds: impl Into<Rect>) -> Layout {
        Layout {
            bounds: bounds.into(),
            direction: Direction::RightToLeft,
            cursor: 0.0
        }
    }

    pub fn new(bounds: impl Into<Rect>, direction: Direction) -> Self {
        Layout {
            bounds: bounds.into(),
            direction: direction,
            cursor: 0.0
        }
    }

    pub fn with_cursor(mut self, cursor: f32) -> Self {
        self.cursor = cursor;
        self
    }
}

impl Layout {
    pub fn push_size(&mut self, size: Vector2) -> Rect {
        match &self.direction {
            Direction::LeftToRight => {
                let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
                self.cursor += size.x;
                let max = (self.bounds.min.x + self.cursor, self.bounds.max.y).into();
                Rect::from_min_max(min, max)
            },
            Direction::RightToLeft => {
                let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
                self.cursor += size.x;
                let min = (self.bounds.max.x - self.cursor, self.bounds.min.y).into();
                Rect::from_min_max(min, max)
            },
            Direction::UpToDown => {
                let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
                self.cursor += size.y;
                let min = (self.bounds.min.x, self.bounds.max.y - self.cursor).into();
                Rect::from_min_max(min, max)
            },
            Direction::DownToUp => {
                let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
                self.cursor += size.y;
                let max = (self.bounds.max.x, self.bounds.min.y + self.cursor).into();
                Rect::from_min_max(min, max)
            },
        }
    }

    pub fn space_left(&self) -> Vector2 {
        match &self.direction {
            Direction::LeftToRight|Direction::RightToLeft => (self.bounds.width() - self.cursor, self.bounds.height()).into(),
            Direction::UpToDown|Direction::DownToUp => (self.bounds.width(), self.bounds.height() - self.cursor).into(),
        }
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn available_rect(&self) -> Rect {
        match &self.direction {
            Direction::LeftToRight => {
                let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
                let max = (self.bounds.min.x + self.cursor + self.space_left().x, self.bounds.max.y).into();
                Rect::from_min_max(min, max)
            },
            Direction::RightToLeft => {
                let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
                let min = (self.bounds.max.x - (self.cursor + self.space_left().x), self.bounds.min.y).into();
                Rect::from_min_max(min, max)
            },
            Direction::UpToDown => {
                let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
                let min = (self.bounds.min.x, self.bounds.max.y - (self.cursor + self.space_left().y)).into();
                Rect::from_min_max(min, max)
            },
            Direction::DownToUp => {
                let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
                let max = (self.bounds.max.x, self.bounds.min.y + (self.cursor + self.space_left().y)).into();
                Rect::from_min_max(min, max)
            },
        }
    }

    pub fn cursor(&self) -> f32 {
        self.cursor
    }
}