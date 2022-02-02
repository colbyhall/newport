use {
	engine::{
		define_run_module,
		Builder,
		Module,
	},
	game2d::Game,
};

pub struct Orchard;
impl Module for Orchard {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Game>()
	}
}

define_run_module!(Orchard, "Orchard");
