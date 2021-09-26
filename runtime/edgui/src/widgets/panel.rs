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
}

impl Panel {
	pub fn top(id: impl ToId) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::TopToBottom,
		}
	}

	pub fn bottom(id: impl ToId) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::BottomToTop,
		}
	}

	pub fn center(id: impl ToId) -> Self {
		Self {
			id: id.to_id(),
			direction: Direction::TopToBottom,
		}
	}
}

impl Panel {
	pub fn build(self, ctx: &mut Context, contents: impl FnOnce(&mut Gui)) {
		let bounds = ctx.canvas();

		let mut gui = ctx.builder(self.id, bounds, Layout::new(self.direction));
		contents(&mut gui);
		let color = gui.style().theme.window_background;
		let used = gui.placer.used;
		gui.painter()
			.insert_shape(0, Shape::solid_rect(used, color, 0.0));
		gui.finish();
	}
}
