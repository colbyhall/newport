use {
	editor::Editor,
	engine::{
		define_run_module,
		Builder,
		Module,
	},
};

pub struct Orchard;
impl Module for Orchard {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Editor>()
	}
}

define_run_module!(Orchard, "Orchard");
