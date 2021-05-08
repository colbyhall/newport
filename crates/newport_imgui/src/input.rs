use crate::math::{ Rect, Vector2 };
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

#[derive(Clone)]
pub struct InputState {
    pub mouse_location: Option<Vector2>,
    pub last_mouse_location: Option<Vector2>,

    pub dt:  f32,
    pub dpi: f32,

    pub key_pressed: [bool; 256],
    pub key_down: [bool; 256],
    pub last_key_down: [bool; 256],

    pub mouse_button_down: [bool; 3],
    pub last_mouse_button_down: [bool; 3],

    pub scroll: f32,

    pub viewport: Rect,

    pub text_input: String,
}

impl InputState {
    pub fn was_key_pressed(&self, key: Input) -> bool {
        self.key_pressed[key.as_key().0 as usize]
    }

    pub fn was_key_released(&self, key: Input) -> bool {
        !self.key_down[key.as_key().0 as usize] && self.last_key_down[key.as_key().0 as usize]
    }

    pub fn was_primary_clicked(&self) -> bool {
        self.mouse_button_down[0] && !self.last_mouse_button_down[0]
    }

    pub fn was_middle_clicked(&self) -> bool {
        self.mouse_button_down[1] && !self.last_mouse_button_down[0]
    }

    pub fn was_secondary_clicked(&self) -> bool {
        self.mouse_button_down[2] && !self.last_mouse_button_down[0]
    }

    pub fn was_primary_released(&self) -> bool {
        !self.mouse_button_down[0] && self.last_mouse_button_down[0]
    }

    pub fn was_middle_released(&self) -> bool {
        !self.mouse_button_down[1] && self.last_mouse_button_down[0]
    }

    pub fn was_secondary_released(&self) -> bool {
        !self.mouse_button_down[2] && self.last_mouse_button_down[0]
    }

    pub fn mouse_is_over(&self, rect: Rect) -> bool {
        match self.mouse_location {
            Some(loc) => rect.point_overlap(loc),
            None => false
        }
    }

    pub fn mouse_move_delta(&self) -> Vector2 {
        if self.mouse_location.is_none() || self.last_mouse_location.is_none() {
            return Vector2::ZERO;
        }

        self.mouse_location.unwrap() - self.last_mouse_location.unwrap()
    }
}