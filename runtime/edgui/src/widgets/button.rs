use crate::{
	Alignment,
	Gui,
	Id,
	Response,
	Sense,
	ToId,
	Widget,
};

use math::Rect;

pub struct Button {
	id: Id,
	label: String,
}

impl Button {
	pub fn new(label: impl ToString) -> Self {
		let label = label.to_string();

		Self {
			id: Id::from(&label),
			label,
		}
	}

	pub fn id(mut self, id: impl ToId) -> Self {
		self.id = id.to_id();
		self
	}
}

impl Widget for Button {
	fn add(self, gui: &mut Gui) -> Response {
		let style = gui.style();

		let text_size = style.text_size(&self.label, None);
		let (bounds, response) = gui.allocate_space(
			text_size + style.button_padding * 2.0,
			Sense::click(self.id),
		);

		let background_color = if gui.is_focused(self.id) {
			style.theme.focused_button
		} else if gui.is_hovered(self.id) {
			style.theme.hovered_button
		} else {
			style.theme.unhovered_button
		};

		gui.painter().push_rect(bounds, background_color, 0.0);
		gui.painter().push_text(
			self.label,
			Rect::from_center(bounds.center(), text_size),
			&style.font_collection,
			style.text_size,
			style.theme.text,
			Alignment::Center,
		);

		response
	}
}
