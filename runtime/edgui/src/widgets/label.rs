use crate::Sense;
use crate::{
	Alignment,
	ColorStyle,
	Gui,

	Response,
	TextStyle,
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
		let color: ColorStyle = gui.style().get();
		let text: TextStyle = gui.style().get();
		// let layout_style: LayoutStyle = gui.style().get();

		let label_rect = text.string_rect(&self.label, text.label_size, None).size();
		let (bounds, response) = gui.allocate_bounds(None, label_rect, Sense::hover());

		gui.painter().push_text(
			self.label,
			bounds,
			&text.font,
			text.label_size,
			color.inactive_foreground,
			Alignment::Center,
		);
		response
	}
}
