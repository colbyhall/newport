use newport::{
	engine,
	game::Game,
};

use engine::{
	Engine,
	Builder,
	Module,
};

struct GameExample;

// Implement the module trait
impl Module for GameExample {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Game>()
	}
}

// Start the app runner
fn main() {
	Engine::builder()
		.module::<GameExample>()
		.name("Game Example")
		.run();
}
