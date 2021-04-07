use newport_math::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
    UpToDown,
    DownToUp,
}

pub struct Layout {
    direction: Direction,
    current:   f32,
    bounds:    Rect,
}