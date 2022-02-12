use engine::{
	define_run_module,
	Builder,
	Module,
};
use game3d::Game;
use physics3d::Physics;

pub struct FirstPerson;
impl Module for FirstPerson {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder.module::<Game>().module::<Physics>()
	}
}

define_run_module!(FirstPerson, "First Person Example");
