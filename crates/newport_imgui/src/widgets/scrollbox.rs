use crate::{
	button_control,
	math,
	Builder,
	ColorStyle,
	Direction,
	Id,
	Layout,
	Retained,
	Shape,
	ToId,
};

use math::{
	InterpTo,
	Rect,
};

pub struct Scrollbox {
	id: Id,
	bounds: Rect,
	direction: Direction,
}

impl Scrollbox {
	pub const BAR_SIZE: f32 = 15.0;
}

#[derive(Default, Clone)]
struct ScrollboxRetained {
	current_scroll: f32,
	target_scroll: f32,

	last_used: f32,
}

impl Retained for ScrollboxRetained {}

impl Scrollbox {
	pub fn new(id: impl ToId, bounds: Rect, direction: Direction) -> Self {
		Self {
			id: id.to_id(),
			bounds: bounds,
			direction: direction,
		}
	}
}

impl Scrollbox {
	pub fn build(self, builder: &mut Builder, contents: impl FnOnce(&mut Builder)) {
		let color: ColorStyle = builder.style().get();
		builder.painter.push_shape(Shape::solid_rect(
			self.bounds,
			color.inactive_background,
			0.0,
		));

		let mut retained = builder.retained::<ScrollboxRetained>(self.id);
		let available = match self.direction {
			Direction::LeftToRight | Direction::RightToLeft => self.bounds.width(),
			Direction::UpToDown | Direction::DownToUp => self.bounds.height(),
		};
		let max_scroll = retained.last_used - available;

		let bounds = if retained.last_used > available {
			let mut bounds = self.bounds;

			let mut scroll_space = match self.direction {
				Direction::LeftToRight | Direction::RightToLeft => {
					bounds.split_bottom(Self::BAR_SIZE)
				}
				Direction::UpToDown | Direction::DownToUp => bounds.split_right(Self::BAR_SIZE),
			};

			let scrollbar_size = available / retained.last_used * available;
			let max_scroll_bar = match self.direction {
				Direction::LeftToRight | Direction::RightToLeft => scroll_space.width(),
				Direction::UpToDown | Direction::DownToUp => scroll_space.height(),
			} - scrollbar_size;

			let is_focused = builder.is_focused(self.id);
			if is_focused {
				let mouse_move = match self.direction {
					Direction::LeftToRight => -builder.input().mouse_move_delta().x,
					Direction::RightToLeft => builder.input().mouse_move_delta().x,
					Direction::UpToDown => -builder.input().mouse_move_delta().y,
					Direction::DownToUp => builder.input().mouse_move_delta().y,
				};

				let scale = max_scroll / max_scroll_bar;

				retained.target_scroll = (retained.target_scroll + mouse_move * scale)
					.min(max_scroll)
					.max(0.0);
				retained.current_scroll = retained.target_scroll;
			}

			// Mouse scroll input which is in content space
			retained.target_scroll = (retained.target_scroll + builder.input().scroll)
				.min(max_scroll)
				.max(0.0);

			// Interp the current scroll to its target
			retained.current_scroll =
				retained
					.current_scroll
					.interp_to(retained.target_scroll, builder.input().dt, 10.0);

			let normalized_scroll = retained.current_scroll / max_scroll;
			let offset = normalized_scroll * max_scroll_bar;

			let scroll_bounds = match self.direction {
				Direction::LeftToRight => {
					scroll_space.split_left(offset);
					scroll_space.split_left(scrollbar_size)
				}
				Direction::RightToLeft => {
					scroll_space.split_right(offset);
					scroll_space.split_right(scrollbar_size)
				}
				Direction::UpToDown => {
					scroll_space.split_top(offset);
					scroll_space.split_top(scrollbar_size)
				}
				Direction::DownToUp => {
					scroll_space.split_bottom(offset);
					scroll_space.split_bottom(scrollbar_size)
				}
			};

			button_control(self.id, scroll_bounds, builder);

			let is_focused = builder.is_focused(self.id);
			let is_hovered = builder.is_hovered(self.id);

			let background_color = if is_focused {
				color.focused_background
			} else if is_hovered {
				color.hovered_background
			} else {
				color.unhovered_background
			};

			builder
				.painter
				.push_shape(Shape::solid_rect(scroll_bounds, background_color, 0.0));

			bounds
		} else {
			self.bounds
		};

		let layout = builder.layout(
			Layout::new(bounds, self.direction).with_cursor(-retained.current_scroll),
			|builder| {
				contents(builder);
			},
		);

		retained.last_used = layout.cursor() + retained.current_scroll;

		builder.set_retained(self.id, retained);
	}
}
