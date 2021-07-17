use crate::{
	math,
	Alignment,
	Builder,
	ColorStyle,
	LayoutStyle,

	Shape,
	TextStyle,
};

use math::{
	Rect,
	Vector2,
};

pub struct Label {
	label: String,
}

impl Label {
	pub fn new(label: impl Into<String>) -> Self {
		let label = label.into();

		Self { label: label }
	}
}

impl Label {
	pub fn build(self, builder: &mut Builder) {
		let color: ColorStyle = builder.style().get();
		let text: TextStyle = builder.style().get();
		let layout_style: LayoutStyle = builder.style().get();

		let label_rect = text.string_rect(&self.label, text.label_size, None).size();
		let bounds = builder.content_bounds(label_rect);

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

		builder.painter.push_shape(Shape::text(
			self.label,
			at,
			&text.font,
			text.label_size,
			builder.input().dpi,
			color.inactive_foreground,
		));
	}
}
