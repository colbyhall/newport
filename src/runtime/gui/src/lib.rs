use {
	derive::Inherit,
	graphics::Painter,
	std::{
		cell::RefCell,
		convert::Into,
		fmt::Debug,
		rc::Rc,
	},
};

#[derive(Debug, Inherit, Clone)]
struct WidgetRef {
	base: Rc<RefCell<Box<dyn Widget>>>,
}

impl<T: Widget> From<T> for WidgetRef {
	fn from(t: T) -> Self {
		Self {
			base: Rc::new(RefCell::new(Box::new(t))),
		}
	}
}

impl WidgetRef {
	fn new(widget: impl Widget) -> Self {
		Self {
			base: Rc::new(RefCell::new(Box::new(widget))),
		}
	}
}

trait Widget: Debug + 'static {
	fn paint(&self, painter: &mut Painter) {}

	fn parent(&self) -> Option<&WidgetRef> {
		None
	}
	fn set_parent(&mut self, parent: Option<WidgetRef>) {}

	fn children(&self, index: usize) -> &[WidgetRef] {
		&[]
	}
	fn set_child(&mut self, index: usize, child: Option<WidgetRef>) {}
}

#[derive(Debug, Default)]
struct Base {
	parent: Option<WidgetRef>,
}

impl Base {
	fn parent(&self) -> Option<&WidgetRef> {
		self.parent.as_ref()
	}
	fn set_parent(&mut self, parent: Option<WidgetRef>) {
		self.parent = parent
	}
}

#[derive(Inherit, Debug)]
struct TextBlock {
	base: Base,

	text: String,
}

impl TextBlock {
	fn new(text: impl Into<String>) -> Self {
		Self {
			base: Base::default(),
			text: text.into(),
		}
	}
}

impl Widget for TextBlock {}

#[derive(Inherit, Debug, Default)]
struct CompoundWidget {
	base: Base,
	child: Option<WidgetRef>,
}

impl CompoundWidget {
	fn set_child(&mut self, child: Option<WidgetRef>) {
		self.child = child;
	}
}

#[derive(Inherit, Debug)]
struct Button {
	base: CompoundWidget,
}

impl Widget for Button {}

#[derive(Inherit, Debug)]
struct Border {
	base: CompoundWidget,
}

impl Widget for Border {}

struct Builder {}

#[test]
fn test() {}
