use crate::{
	Id,
	Sense,
	NUM_MOUSE_BUTTONS,
};

use math::Rect;

#[derive(Copy, Clone, PartialEq)]
pub struct Response {
	pub id: Option<Id>,
	pub bounds: Rect,
	pub sense: Sense,

	pub(crate) clicked: [bool; NUM_MOUSE_BUTTONS],
	pub(crate) dragged: [bool; NUM_MOUSE_BUTTONS],
	pub(crate) hovered: bool,
}

impl Response {
	pub fn none() -> Self {
		Self {
			id: None,
			bounds: Rect::ZERO,
			sense: Sense::hover(),

			clicked: [false; NUM_MOUSE_BUTTONS],
			dragged: [false; NUM_MOUSE_BUTTONS],
			hovered: false,
		}
	}

	pub fn clicked(self) -> bool {
		self.clicked[0]
	}
}
