mod button;
// mod checkbox;
mod label;
mod panel;
mod view;
// mod scrollbox;
// mod text_edit;

use crate::{
	Gui,
	Response,
};

pub use {
	button::*,
	// checkbox::*,
	label::*,
	panel::*,
	view::*,
	// scrollbox::*,
	// text_edit::*,
};

pub trait Widget {
	fn add(self, gui: &mut Gui) -> Response;
}
