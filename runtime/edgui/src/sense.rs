use crate::Id;

// This is copied from egui but modified for my needs

/// What sort of interaction is a widget sensitive to?
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sense {
	pub id: Option<Id>,

	/// buttons, sliders, windows ...
	pub click: bool,

	/// sliders, windows, scroll bars, scroll areas ...
	pub drag: bool,

	/// this widgets want focus.
	/// Anything interactive + labels that can be focused
	/// for the benefit of screen readers.
	pub focusable: bool,
}

impl Sense {
	/// Senses no clicks or drags. Only senses mouse hover.
	pub fn hover() -> Self {
		Self {
			id: None,

			click: false,
			drag: false,
			focusable: false,
		}
	}

	/// Senses no clicks or drags, but can be focused with the keyboard.
	/// Used for labels that can be focused for the benefit of screen readers.
	pub fn focusable_noninteractive(id: Id) -> Self {
		Self {
			id: Some(id),
			click: false,
			drag: false,
			focusable: true,
		}
	}

	/// Sense clicks and hover, but not drags.
	pub fn click(id: Id) -> Self {
		Self {
			id: Some(id),
			click: true,
			drag: false,
			focusable: true,
		}
	}

	/// Sense drags and hover, but not clicks.
	pub fn drag(id: Id) -> Self {
		Self {
			id: Some(id),
			click: false,
			drag: true,
			focusable: true,
		}
	}

	/// Sense both clicks, drags and hover (e.g. a slider or window).
	pub fn click_and_drag(id: Id) -> Self {
		Self {
			id: Some(id),
			click: true,
			drag: true,
			focusable: true,
		}
	}

	/// Returns true if we sense either clicks or drags.
	pub fn interactive(&self) -> bool {
		self.click || self.drag
	}
}
