use std::fmt::Debug;

use graphics::{
	FontCollection,
	PainterStyle,
};
use resources::Handle;

use {
	derive::Widget,
	graphics::Painter,
	math::{
		Color,
		Rect,
	},
	std::{
		any::Any,
		convert::Into,
		fmt,
	},
};

#[derive(Clone, Copy, PartialEq)]
pub struct WidgetRef {
	pub generation: u32,
	pub index: u32,
}

impl WidgetRef {
	pub fn merged(self) -> u64 {
		((self.generation as u64) << 32) | (self.index as u64)
	}
}

impl fmt::Debug for WidgetRef {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match crate::Gui::widgets().find(*self) {
			Some(me) => {
				f.write_fmt(format_args!("(0x{:X}) ", self.merged()))?;
				me.fmt(f)
			}
			None => Ok(()),
		}
	}
}

pub trait Widget: fmt::Debug + 'static {
	fn parent(&self) -> Option<WidgetRef>;
	fn set_parent(&mut self, parent: Option<WidgetRef>);

	fn slot(&self) -> Option<&dyn Slot>;
	fn slot_mut(&mut self) -> Option<&mut dyn Slot>;

	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;

	fn paint(&self, painter: &mut Painter, bounds: Rect);
}

pub trait Slot: fmt::Debug + 'static {
	fn children(&self) -> &[WidgetRef];
	fn set_child(&mut self, index: usize, widget: Option<WidgetRef>);
}

struct Entry {
	generation: u32,
	widget: Box<dyn Widget>,
}

#[derive(Default)]
pub struct WidgetTree {
	slots: Vec<Entry>,
	free: Vec<usize>,
	base: Option<WidgetRef>,
}

impl WidgetTree {
	pub fn new() -> Self {
		Self {
			slots: Vec::with_capacity(1024),
			free: Vec::with_capacity(256),
			base: None,
		}
	}

	pub fn create(&mut self, widget: impl Widget) -> WidgetRef {
		let result = match self.free.pop() {
			Some(index) => {
				let entry = &mut self.slots[index];
				entry.widget = Box::new(widget);
				WidgetRef {
					index: index as u32,
					generation: entry.generation,
				}
			}
			None => {
				let index = self.slots.len();
				self.slots.push(Entry {
					generation: 0,
					widget: Box::new(widget),
				});
				WidgetRef {
					index: index as u32,
					generation: 0,
				}
			}
		};

		let children = match self.find(result).unwrap().slot() {
			Some(slot) => slot.children(),
			None => &[],
		}
		.to_vec();
		children.iter().for_each(|c| {
			if let Some(child) = self.find_mut(*c) {
				child.set_parent(Some(result));
			}
		});

		result
	}

	pub fn destroy(&mut self, widget: WidgetRef) -> bool {
		let entry = self.slots.get_mut(widget.index as usize);
		if let Some(entry) = entry {
			if entry.generation == widget.generation {
				entry.generation += 1;
				self.free.push(widget.index as usize);
				return true;
			}
		}
		false
	}

	pub fn find(&self, widget: WidgetRef) -> Option<&dyn Widget> {
		self.slots
			.get(widget.index as usize)
			.map(|e| e.widget.as_ref())
	}

	pub fn find_mut(&mut self, widget: WidgetRef) -> Option<&mut dyn Widget> {
		self.slots
			.get_mut(widget.index as usize)
			.map(|e| e.widget.as_mut())
	}

	pub fn base(&self) -> Option<WidgetRef> {
		self.base
	}

	pub fn set_base(&mut self, base: Option<WidgetRef>) {
		self.base = base;
		crate::Gui::layouts_mut().rebuild();
	}
}

impl fmt::Debug for WidgetTree {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.base.fmt(f)
	}
}

///////////////////////////////////////////////////////////////////////////////////////

#[derive(Widget)]
pub struct TextBlock {
	parent: Option<WidgetRef>,
	text: String,

	color: Color,
	size: u32,
	font: Handle<FontCollection>,
}

impl TextBlock {
	pub fn new(text: impl Into<String>) -> Self {
		Self {
			parent: None,
			text: text.into(),

			color: Color::WHITE,
			size: 12,
			font: Handle::default(),
		}
	}

	fn paint(&self, painter: &mut Painter, bounds: Rect) {}
}

#[derive(Default, Widget)]
pub struct Button {
	parent: Option<WidgetRef>,
	slot: CompoundSlot,

	normal: Color,
	hovered: Color,
	focused: Color,
}

impl Button {
	pub fn new(slot: impl FnOnce(CompoundSlot) -> CompoundSlot) -> Self {
		Self {
			parent: None,
			slot: (slot)(CompoundSlot::new()),

			normal: Color::WHITE,  // TODO: Styling
			hovered: Color::GREEN, // TODO: Styling
			focused: Color::RED,   // TODO: Styling
		}
	}

	fn paint(&self, painter: &mut Painter, bounds: Rect) {
		let style = PainterStyle {
			color: self.hovered,
			..Default::default()
		};
		painter.fill_rect(&style, bounds);
	}
}

#[derive(Default, Widget)]
pub struct Border {
	parent: Option<WidgetRef>,
	slot: CompoundSlot,

	color: Color,
}

impl Border {
	pub fn new(slot: impl FnOnce(CompoundSlot) -> CompoundSlot) -> Self {
		Self {
			parent: None,
			slot: (slot)(CompoundSlot::new()),

			color: Color::WHITE, // TODO: Styling
		}
	}

	fn paint(&self, painter: &mut Painter, bounds: Rect) {
		let style = PainterStyle {
			color: self.color,
			..Default::default()
		};
		painter.fill_rect(&style, bounds);
	}
}

///////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct CompoundSlot {
	child: Option<WidgetRef>,
}

impl CompoundSlot {
	pub fn new() -> Self {
		Self { child: None }
	}

	pub fn child(mut self, widget: impl Widget) -> Self {
		self.child = Some(crate::Gui::widgets_mut().create(widget));
		self
	}
}

impl Slot for CompoundSlot {
	fn children(&self) -> &[WidgetRef] {
		match &self.child {
			Some(child) => std::slice::from_ref(child),
			None => &[],
		}
	}
	fn set_child(&mut self, index: usize, widget: Option<WidgetRef>) {
		assert_eq!(index, 0, "This slot only supports a single child.");
		self.child = widget
	}
}
