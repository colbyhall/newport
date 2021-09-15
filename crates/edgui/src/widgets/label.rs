use crate::{
	Alignment,
	ColorStyle,
	Gui,
	LayoutStyle,

	Response,
	Shape,
	TextStyle,
	Widget,
};

use math::{
	Rect,
	Vector2,
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
		let layout_style: LayoutStyle = gui.style().get();

		let label_rect = text.string_rect(&self.label, text.label_size, None).size();
		let bounds = gui.content_bounds(label_rect);

		let at = match text.alignment {
			Alignment::Left => {
				bounds.top_left()
					+ Vector2::new(
						layout_style.padding.top_left().x,
						-layout_style.padding.top_left().y,
					)
			}
			Alignment::Center => Rect::from_pos_size(bounds.pos(), label_rect).top_left(),
			Alignment::Right => bounds.top_right() - layout_style.padding.top_right(),
		};

		gui.painter.push_shape(Shape::text(
			self.label,
			at,
			&text.font,
			text.label_size,
			gui.input().dpi,
			color.inactive_foreground,
		));

		Response::none(bounds)
	}
}
