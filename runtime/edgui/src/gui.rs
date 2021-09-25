use crate::{
	Button,
	Context,
	Id,
	InputState,
	Label,
	Layout,
	Painter,
	Placer,
	Response,
	Retained,
	Sense,
	Style,
	Widget,
};

use math::{
	Rect,
	Vector2,
};

pub struct Gui<'a> {
	pub(crate) id: Id,
	pub(crate) placer: Placer,

	pub(crate) painter: Painter,
	pub(crate) context: &'a mut Context,
}

/// # General API
impl<'a> Gui<'a> {
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn painter(&mut self) -> &mut Painter {
		&mut self.painter
	}

	pub fn input(&self) -> &InputState {
		&self.context.input
	}

	pub fn finish(self) {
		self.context.push_layer(self.painter)
	}

	pub fn retained<T: Retained + Default + Clone>(&mut self, id: Id) -> T {
		self.context.retained::<T>(id)
	}

	pub fn set_retained<T: Retained>(&mut self, id: Id, t: T) {
		self.context.set_retained(id, t);
	}

	pub fn style(&self) -> Style {
		self.context.style.clone()
	}
}

/// # Interaction
impl<'a> Gui<'a> {
	pub fn is_focused(&self, id: Id) -> bool {
		self.context.is_focused(id)
	}

	pub fn is_hovered(&self, id: Id) -> bool {
		self.context.is_hovered(id)
	}

	pub fn interact(&mut self, bounds: Rect, sense: Sense) -> Response {
		self.context.interact(self.placer.bounds(), bounds, sense)
	}
}

impl<'a> Gui<'a> {
	pub fn add(&mut self, widget: impl Widget) -> Response {
		widget.add(self)
	}

	pub fn label(&mut self, text: impl ToString) {
		self.add(Label::new(text));
	}

	#[must_use]
	pub fn button(&mut self, text: impl ToString) -> Response {
		self.add(Button::new(text))
	}
}

impl<'a> Gui<'a> {
	pub fn layout(&mut self, layout: Layout, content: impl FnOnce(&mut Gui)) {
		let current = self.placer;
		self.placer = Placer::new(self.available_rect(), layout);
		self.painter.push_scissor(self.placer.bounds());
		content(self);
		self.painter.pop_scissor();
		self.placer = current;
	}

	pub fn available_rect(&self) -> Rect {
		self.placer.available_rect()
	}

	pub fn allocate_desired(&mut self, desired: Vector2, sense: Sense) -> (Rect, Response) {
		let size = desired; // TODO: Wrapping and justified

		let style = self.style();
		let layout_rect = self.placer.push_size(size + style.margin * 2.0);
		let bounds = Rect::from_center(layout_rect.center(), size);

		(bounds, self.interact(bounds, sense))
	}

	pub fn add_spacing(&mut self, amount: f32) {
		self.placer.push_size(Vector2::new(amount, amount));
	}

	pub fn horizontal(&mut self, content: impl FnOnce(&mut Gui)) {
		self.layout(Layout::left_to_right(), content)
	}
}
