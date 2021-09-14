use crate::{
	Button,
	ButtonResponse,
	Context,
	Id,
	InputState,
	Label,
	Layout,
	LayoutStyle,
	Painter,
	Retained,
	Sizing,
	Style,
	StyleMap,
	TextStyle,
};

use math::{
	Rect,
	Vector2,
};

pub struct Builder<'a> {
	pub id: Id,
	pub layout: Layout,

	pub painter: Painter,
	pub(crate) context: &'a mut Context,
}

impl<'a> Builder<'a> {
	pub fn finish(self) {
		self.context.push_layer(self.painter)
	}

	pub fn input(&self) -> &InputState {
		&self.context.input
	}

	pub fn is_focused(&self, id: Id) -> bool {
		match &self.context.focused {
			Some(focused) => *focused == id,
			None => false,
		}
	}

	pub fn is_hovered(&self, id: Id) -> bool {
		match &self.context.hovered {
			Some(hovered) => *hovered == id,
			None => false,
		}
	}

	pub fn focus(&mut self, id: Id) {
		self.context.focused = Some(id);
	}

	pub fn unfocus(&mut self, id: Id) -> bool {
		if self.is_focused(id) {
			self.context.focused = None;
			return true;
		}
		false
	}

	pub fn hover(&mut self, id: Id) {
		self.context.hovered = Some(id);
	}

	pub fn unhover(&mut self, id: Id) -> bool {
		if self.is_hovered(id) {
			self.context.hovered = None;
			return true;
		}
		false
	}

	#[must_use = "If a response is not being used then use a label"]
	pub fn button(&mut self, label: impl Into<String>) -> ButtonResponse {
		Button::new(label).build(self)
	}

	pub fn label(&mut self, label: impl Into<String>) {
		Label::new(label).build(self)
	}

	pub fn layout(&mut self, layout: Layout, content: impl FnOnce(&mut Builder)) -> Layout {
		let current = self.layout;
		self.layout = layout;
		self.painter.push_scissor(layout.bounds());
		content(self);
		self.painter.pop_scissor();
		let result = self.layout;
		self.layout = current;
		result
	}

	pub fn available_rect(&self) -> Rect {
		self.layout.available_rect()
	}

	pub fn content_bounds(&mut self, space_needed: Vector2) -> Rect {
		let style: LayoutStyle = self.style().get();

		let space_available = self.layout.space_left();
		let content_size = style.content_size(space_needed, space_available);

		let layout_rect = self.layout.push_size(style.spacing_size(content_size));

		Rect::from_pos_size(layout_rect.pos(), content_size)
	}

	pub fn add_spacing(&mut self, amount: f32) {
		self.layout.push_size(Vector2::new(amount, amount));
	}

	pub fn retained<T: Retained + Default + Clone>(&mut self, id: Id) -> T {
		self.context.retained::<T>(id)
	}

	pub fn set_retained<T: Retained>(&mut self, id: Id, t: T) {
		self.context.set_retained(id, t);
	}

	pub fn style(&mut self) -> &mut StyleMap {
		&mut self.context.style
	}

	pub fn scoped_style<T: Style>(&mut self, in_style: T, contents: impl FnOnce(&mut Builder)) {
		self.style().push(in_style);
		contents(self);
		self.style().pop::<T>();
	}

	pub fn label_height_with_padding(&mut self) -> f32 {
		let layout_style: LayoutStyle = self.style().get();
		let text_style: TextStyle = self.style().get();

		text_style.label_height() + layout_style.padding.min.y + layout_style.padding.max.y
	}

	pub fn fill(&mut self, contents: impl FnOnce(&mut Builder)) {
		let mut layout_style: LayoutStyle = self.style().get();
		layout_style.width_sizing = Sizing::Fill;
		self.scoped_style(layout_style, contents);
	}
}
