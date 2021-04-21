use crate::math::Rect;
pub use crate::os::window::WindowEvent as Event;
pub use crate::os::input::*;

use std::collections::VecDeque;

#[derive(Default, Clone)]
pub struct RawInput {
    pub viewport: Rect,

    pub dt:  f32,
    pub dpi: f32,

    pub events: VecDeque<Event>,
}
