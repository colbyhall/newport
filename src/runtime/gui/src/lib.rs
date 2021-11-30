use {
	derive::Widget,
	std::{
		cell::RefCell,
		convert::Into,
		fmt::Debug,
		rc::Rc,
	},
};

#[derive(Debug, Clone)]
pub struct WidgetRef(Rc<RefCell<Box<dyn Widget>>>);

impl WidgetRef {
	pub fn new(widget: impl Widget) -> Self {
		let result = Self(Rc::new(RefCell::new(Box::new(widget))));
		{
			let borrowed = result.borrow();
			if let Some(slot) = borrowed.slot() {
				slot.children()
					.iter()
					.for_each(|c| c.borrow_mut().set_parent(Some(&result)))
			}
		}
		result
	}
}

impl std::ops::Deref for WidgetRef {
	type Target = Rc<RefCell<Box<dyn Widget>>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl std::ops::DerefMut for WidgetRef {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub trait Widget: Debug + 'static {
	fn parent(&self) -> Option<&WidgetRef>;
	fn set_parent(&mut self, parent: Option<&WidgetRef>);

	fn slot(&self) -> Option<&dyn Slot>;
	fn slot_mut(&mut self) -> Option<&mut dyn Slot>;
}

#[derive(Debug, Widget)]
pub struct TextBlock {
	parent: Option<WidgetRef>,
	text: String,
}

impl TextBlock {
	pub fn new(text: impl Into<String>) -> Self {
		Self {
			parent: None,
			text: text.into(),
		}
	}
}

#[derive(Default, Debug, Widget)]
pub struct Button {
	parent: Option<WidgetRef>,
	slot: CompoundSlot,
}

impl Button {
	pub fn new(slot: impl FnOnce(CompoundSlot) -> CompoundSlot) -> Self {
		Self {
			parent: None,
			slot: (slot)(CompoundSlot::new()),
		}
	}
}

#[derive(Default, Debug, Widget)]
pub struct Border {
	parent: Option<WidgetRef>,
	slot: CompoundSlot,
}

impl Border {
	pub fn new(slot: impl FnOnce(CompoundSlot) -> CompoundSlot) -> Self {
		Self {
			parent: None,
			slot: (slot)(CompoundSlot::new()),
		}
	}
}

pub trait Slot: Debug + 'static {
	fn children(&self) -> &[WidgetRef];
	fn set_child(&mut self, index: usize, widget: &WidgetRef);
}

#[derive(Default, Debug)]
pub struct CompoundSlot {
	child: Option<WidgetRef>,
}

impl CompoundSlot {
	pub fn new() -> Self {
		Self { child: None }
	}

	pub fn child(mut self, widget: impl Widget) -> Self {
		self.child = Some(WidgetRef::new(widget));
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
	fn set_child(&mut self, index: usize, widget: &WidgetRef) {
		assert_eq!(index, 0, "This slot only supports a single child.");
		self.child = Some(widget.clone())
	}
}

#[test]
fn test() {
	let button = Border::new(|slot| {
		slot.child(Button::new(|slot| {
			slot.child(TextBlock::new("Hello World"))
		}))
	});
	println!("{:?}", button);
}
