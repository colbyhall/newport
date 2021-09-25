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
	size: f32,
}

impl Panel {
	pub fn top(id: impl ToId, size: f32) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::TopToBottom,
			size,
		}
	}

	pub fn bottom(id: impl ToId, size: f32) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::BottomToTop,
			size,
		}
	}
}

impl Panel {
	pub fn build(self, ctx: &mut Context, contents: impl FnOnce(&mut Gui)) {
		let bounds = match self.direction {
			Direction::TopToBottom => ctx.split_canvas_top(self.size),
			Direction::BottomToTop => ctx.split_canvas_bottom(self.size),
			_ => unimplemented!(),
		};

		let mut gui = ctx.builder(self.id, bounds, Layout::new(self.direction));
		let color = gui.style().theme.window_background;

		gui.painter()
			.push_shape(Shape::solid_rect(bounds, color, 0.0));
		contents(&mut gui);
		gui.finish();
	}
}
