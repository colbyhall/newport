use crate::{
	Id,
	NUM_MOUSE_BUTTONS,
};

use math::Rect;

pub struct Response {
	pub id: Option<Id>,
	pub rect: Rect,

	pub(crate) clicked: [bool; NUM_MOUSE_BUTTONS],
	pub(crate) hovered: bool,
}

impl Response {
	pub fn none(rect: Rect) -> Self {
		Self {
			id: None,
			rect,

			clicked: [false; NUM_MOUSE_BUTTONS],
			hovered: false,
		}
	}
}
