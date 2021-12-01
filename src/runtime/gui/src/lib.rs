use graphics::Graphics;
use resources::ResourceManager;

mod layout;
mod widget;

pub use {
	layout::*,
	widget::*,
};

use {
	engine::{
		Engine,
		Module,
	},
	std::thread,
};

pub struct Gui {
	thread_id: thread::ThreadId,

	widgets: WidgetTree,
	layouts: LayoutTree,
}

impl Gui {
	fn as_ref<'a>() -> &'a Gui {
		let gui: &'a Gui = Engine::module().unwrap();
		assert_eq!(thread::current().id(), gui.thread_id);
		gui
	}

	fn as_mut<'a>() -> &'a mut Gui {
		let gui: &'a mut Gui = unsafe { Engine::module_mut().unwrap() };
		assert_eq!(thread::current().id(), gui.thread_id);
		gui
	}

	pub fn widgets<'a>() -> &'a WidgetTree {
		&Self::as_ref().widgets
	}

	pub fn widgets_mut<'a>() -> &'a mut WidgetTree {
		&mut Self::as_mut().widgets
	}

	pub fn layouts<'a>() -> &'a LayoutTree {
		&Self::as_ref().layouts
	}

	pub fn layouts_mut<'a>() -> &'a mut LayoutTree {
		&mut Self::as_mut().layouts
	}
}

impl Module for Gui {
	fn new() -> Self {
		Self {
			thread_id: thread::current().id(),

			widgets: WidgetTree::new(),
			layouts: LayoutTree::new(),
		}
	}

	fn depends_on(builder: engine::Builder) -> engine::Builder {
		builder
			.module::<Graphics>()
			.module::<ResourceManager>()
			.display(|| {})
	}
}

#[test]
fn test() {
	Engine::builder().module::<Gui>().test().unwrap();

	let button = Border::new(|slot| {
		slot.child(Button::new(|slot| {
			slot.child(TextBlock::new("Hello World"))
		}))
	});
	let widgets = Gui::widgets_mut();
	let button = widgets.create(button);
	Gui::widgets_mut().set_base(Some(button));
	println!("{:#?}", widgets);
}
