use crate::math::Rect;
use crate::os::window::WindowEvent;

use std::collections::VecDeque;

pub struct Input {
    pub viewport: Rect,

    pub dt:  f32,
    pub dpi: f32,

    pub events: VecDeque<WindowEvent>,
}