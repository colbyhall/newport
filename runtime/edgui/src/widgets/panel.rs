use crate::{
	Context,
	Gui,
	Id,
	Layout,
	Shape,
	ToId,
};

pub enum PanelVariant {
	Top,
	Bottom,
}

pub struct Panel {
	id: Id,
	variant: PanelVariant,
	size: f32,
}

impl Panel {
	pub fn top(id: impl ToId, size: f32) -> Self {
		Self {
			id: id.to_id(),
			variant: PanelVariant::Top,
			size,
		}
	}

	pub fn bottom(id: impl ToId, size: f32) -> Self {
		Self {
			id: id.to_id(),
			variant: PanelVariant::Bottom,
			size,
		}
	}
}

impl Panel {
	pub fn build(self, ctx: &mut Context, contents: impl FnOnce(&mut Gui)) {
		let layout = match self.variant {
			PanelVariant::Top => {
				let rect = ctx.split_canvas_top(self.size);
				Layout::left_to_right(rect)
			}
			PanelVariant::Bottom => {
				let rect = ctx.split_canvas_bottom(self.size);
				Layout::left_to_right(rect)
			}
		};

		let mut gui = ctx.builder(self.id, layout);
		let color = gui.style().theme.window_background;

		gui.painter()
			.push_shape(Shape::solid_rect(layout.bounds(), color, 0.0));
		contents(&mut gui);
		gui.finish();
	}
}
