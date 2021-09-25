use crate::Sense;
use crate::{
	Alignment,
	Gui,

	Response,
	Widget,
};

pub struct Label {
	label: String,
}

impl Label {
	pub fn new(label: impl ToString) -> Self {
		let label = label.to_string();

		Self { label }
	}
}

impl Widget for Label {
	fn add(self, gui: &mut Gui) -> Response {
		let style = gui.style();

		let (bounds, response) =
			gui.allocate_desired(style.text_size(&self.label, None), Sense::hover());

		gui.painter().push_text(
			self.label,
			bounds,
			&style.font_collection,
			style.text_size,
			style.theme.text,
			Alignment::Center,
		);
		response
	}
}
