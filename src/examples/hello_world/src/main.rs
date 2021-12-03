use {
	engine::{
		define_run_module,
		Builder,
		Module,
	},
	gui::*,
};

struct HelloWorld;

impl Module for HelloWorld {
	fn new() -> Self {
		let button = Border::new(|slot| {
			slot.child(Button::new(|slot| {
				slot.child(TextBlock::new("Hello World"))
			}))
		});
		let widgets = Gui::widgets_mut();
		let button = widgets.create(button);
		Gui::widgets_mut().set_base(Some(button));

		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Gui>()
	}
}

define_run_module!(HelloWorld, "Hello World");
