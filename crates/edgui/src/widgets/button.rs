use crate::{
	Builder,
	ColorStyle,
	Id,
	Shape,

	TextStyle,
	ToId,
};

use asset::AssetRef;
use gpu::Texture;
use math::{
	Rect,
	Vector2,
};

pub struct Button {
	id: Id,
	label: String,
}

impl Button {
	pub fn new(label: impl Into<String>) -> Self {
		let label = label.into();

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

impl Button {
	#[must_use = "If a response is not being used then use a label"]
	pub fn build(self, builder: &mut Builder) -> ButtonResponse {
		let color: ColorStyle = builder.style().get();
		let text: TextStyle = builder.style().get();

		let label_rect = text.string_rect(&self.label, text.label_size, None).size();
		let bounds = builder.content_bounds(label_rect);

		let response = button_control(self.id, bounds, builder);

		let is_focused = builder.is_focused(self.id);
		let is_hovered = builder.is_hovered(self.id);

		let (background_color, foreground_color) = {
			let background_color = if is_focused {
				color.focused_background
			} else if is_hovered {
				color.hovered_background
			} else {
				color.unhovered_background
			};

			let foreground_color = if is_focused {
				color.focused_foreground
			} else if is_hovered {
				color.hovered_foreground
			} else {
				color.unhovered_foreground
			};

			(background_color, foreground_color)
		};

		builder
			.painter
			.push_shape(Shape::solid_rect(bounds, background_color, 0.0));

		let at = Rect::from_pos_size(bounds.pos(), label_rect).top_left();
		builder.painter.push_shape(Shape::text(
			self.label,
			at,
			&text.font,
			text.label_size,
			builder.input().dpi,
			foreground_color,
		));

		response
	}
}
