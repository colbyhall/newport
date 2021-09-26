use crate::{
	Context,
	Direction,
	Gui,
	Id,
	Layout,
	Shape,
	ToId,
};

pub struct Panel {
	id: Id,
	direction: Direction,
	size: Option<f32>,
}

impl Panel {
	pub fn top(id: impl ToId, size: f32) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::TopToBottom,
			size: Some(size),
		}
	}

	pub fn bottom(id: impl ToId, size: f32) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::BottomToTop,
			size: Some(size),
		}
	}

	pub fn center(id: impl ToId) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::TopToBottom,
			size: None,
		}
	}
}

impl Panel {
	pub fn build(self, ctx: &mut Context, contents: impl FnOnce(&mut Gui)) {
		let bounds = if let Some(size) = self.size {
			match self.direction {
				Direction::TopToBottom => ctx.split_canvas_top(size),
				Direction::BottomToTop => ctx.split_canvas_bottom(size),
				_ => unimplemented!(),
			}
		} else {
			ctx.take_canvas()
		};

		let mut gui = ctx.builder(self.id, bounds, Layout::new(self.direction));
		let color = gui.style().theme.window_background;

		gui.painter()
			.push_shape(Shape::solid_rect(bounds, color, 0.0));
		contents(&mut gui);
		gui.finish();
	}
}
