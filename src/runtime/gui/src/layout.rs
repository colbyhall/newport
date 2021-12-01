use {
	crate::{
		Gui,
		WidgetRef,
	},
	math::*,
};

#[derive(Clone, Copy, Default)]
pub struct Layout {
	pub local_position: Vector2,
	pub absolute_position: Vector2,
	pub size: Vector2,
}

#[derive(Default)]
pub struct LayoutTree {
	layouts: Vec<Layout>,
}

impl LayoutTree {
	pub fn new() -> Self {
		Self {
			layouts: vec![Default::default(); 1024],
		}
	}

	pub fn find(&self, widget: WidgetRef) -> &Layout {
		&self.layouts[widget.index as usize]
	}

	pub fn find_mut(&mut self, widget: WidgetRef) -> &mut Layout {
		&mut self.layouts[widget.index as usize]
	}

	pub fn rebuild(&mut self) {
		let widgets = Gui::widgets();

		// Idomatic way of garbbing the viewport size
		let viewport = engine::Engine::window().unwrap().inner_size();
		let viewport = Vector2::new(viewport.width as f32, viewport.height as f32);

		let base = widgets.base();
		if base.is_none() {
			return;
		}
		let base = base.unwrap();
		{
			let layout = self.find_mut(base);
			layout.local_position = Vector2::ZERO;
			layout.absolute_position = Vector2::ZERO;
			layout.size = viewport;
		}

		let base = widgets.find(base);
		if base.is_none() {
			return;
		}
		let base = base.unwrap();
	}
}
