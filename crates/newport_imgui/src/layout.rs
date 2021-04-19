use crate::math::Rect;

pub enum Direction {
    LeftToRight,
    RightToLeft,
    UpToDown,
    DownToUp,
}

pub enum Alignment {
    Left,
    Center,
    Right,
}

pub struct Layout {
    bounds:     Rect,
    direction:  Direction,
    cursor:     f32,
}

impl Layout {
    pub fn push_size(&mut self, size: f32) -> Rect {
        match &self.direction {
            Direction::LeftToRight => {
                let min = (self.bounds.min.x + self.cursor, self.bounds.min.y).into();
                self.cursor += size;
                let max = (self.bounds.min.x + self.cursor, self.bounds.max.y).into();
                Rect::from_min_max(min, max)
            },
            Direction::RightToLeft => {
                let max = (self.bounds.max.x - self.cursor, self.bounds.max.y).into();
                self.cursor += size;
                let min = (self.bounds.max.x - self.cursor, self.bounds.min.y).into();
                Rect::from_min_max(min, max)
            },
            Direction::UpToDown => {
                let max = (self.bounds.max.x, self.bounds.max.y - self.cursor).into();
                self.cursor += size;
                let min = (self.bounds.min.x, self.bounds.max.y - self.cursor).into();
                Rect::from_min_max(min, max)
            },
            Direction::DownToUp => {
                let min = (self.bounds.min.x, self.bounds.min.y + self.cursor).into();
                self.cursor += size;
                let max = (self.bounds.max.x, self.bounds.min.y + self.cursor).into();
                Rect::from_min_max(min, max)
            },
        }
    }

    pub fn space_left(&self) -> f32 {
        match &self.direction {
            Direction::LeftToRight|Direction::RightToLeft => self.bounds.width() - self.cursor,
            Direction::UpToDown|Direction::DownToUp => self.bounds.height() - self.cursor,
        }
    }
}