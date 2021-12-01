use {
	derive::Widget,
	engine::{
		Engine,
		Module,
	},
	std::{
		convert::Into,
		fmt,
		fmt::Debug,
	},
};

#[derive(Clone, Copy, PartialEq)]
pub struct WidgetRef {
	index: u32,
	generation: u32,
}

impl fmt::Debug for WidgetRef {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match Gui::find(*self) {
			Some(me) => me.fmt(f),
			None => Ok(()),
		}
	}
}

struct Entry {
	generation: u32,
	widget: Box<dyn Widget>,
}

pub struct Gui {
	widgets: Vec<Entry>,
	free: Vec<usize>,
	base: Option<WidgetRef>,
}

impl Gui {
	pub fn create(widget: impl Widget) -> WidgetRef {
		// TODO: Check if is on the main thread or not

		let gui: &mut Gui = unsafe { Engine::module_mut().unwrap() };
		match gui.free.pop() {
			Some(index) => {
				let entry = &mut gui.widgets[index];
				entry.widget = Box::new(widget);
				WidgetRef {
					index: index as u32,
					generation: entry.generation,
				}
			}
			None => {
				let index = gui.widgets.len();
				gui.widgets.push(Entry {
					generation: 0,
					widget: Box::new(widget),
				});
				WidgetRef {
					index: index as u32,
					generation: 0,
				}
			}
		}
	}

	pub fn destroy(widget: WidgetRef) -> bool {
		let gui: &mut Gui = unsafe { Engine::module_mut().unwrap() };
		let entry = gui.widgets.get_mut(widget.index as usize);
		if let Some(entry) = entry {
			if entry.generation == widget.generation {
				entry.generation += 1;
				return true;
			}
		}
		false
	}

	pub fn find<'a>(widget: WidgetRef) -> Option<&'a dyn Widget> {
		// TODO: Check if is on the main thread or not

		let gui: &Gui = Engine::module().unwrap();
		gui.widgets
			.get(widget.index as usize)
			.map(|e| e.widget.as_ref())
	}

	pub fn find_mut<'a>(widget: WidgetRef) -> Option<&'a mut dyn Widget> {
		// TODO: Check if is on the main thread or not

		let gui: &mut Gui = unsafe { Engine::module_mut().unwrap() };
		gui.widgets
			.get_mut(widget.index as usize)
			.map(|e| e.widget.as_mut())
	}

	pub fn base() -> Option<WidgetRef> {
		// TODO: Check if is on the main thread or not

		let gui: &Gui = Engine::module().unwrap();
		gui.base
	}

	pub fn set_base(base: Option<WidgetRef>) {
		// TODO: Check if is on the main thread or not

		let gui: &mut Gui = unsafe { Engine::module_mut().unwrap() };
		gui.base = base
	}
}

impl Module for Gui {
	fn new() -> Self {
		Self {
			widgets: Vec::with_capacity(1024),
			free: Vec::with_capacity(256),
			base: None,
		}
	}
}

pub trait Widget: Debug + 'static {
	fn parent(&self) -> Option<WidgetRef>;
	fn set_parent(&mut self, parent: Option<WidgetRef>);

	fn slot(&self) -> Option<&dyn Slot>;
	fn slot_mut(&mut self) -> Option<&mut dyn Slot>;
}

#[derive(Widget)]
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

#[derive(Default, Widget)]
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

#[derive(Default, Widget)]
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
	fn set_child(&mut self, index: usize, widget: Option<WidgetRef>);
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
		self.child = Some(Gui::create(widget));
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

#[test]
fn test() {
	Engine::builder().module::<Gui>().spawn().unwrap();

	let button = Border::new(|slot| {
		slot.child(Button::new(|slot| {
			slot.child(TextBlock::new("Hello World"))
		}))
	});
	Gui::set_base(Some(Gui::create(button)));
	println!("{:#?}", Gui::base());
}
