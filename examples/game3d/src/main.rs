use {
	engine::{
		Builder,
		Engine,
		Module,
	},
	game::Game,
};

// First thing first is to define our module struct
struct Game3dExample;

// Implement the module trait
impl Module for Game3dExample {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Game>()
	}
}

// Start the app runner
fn main() -> Result<(), std::io::Error> {
	Engine::builder()
		.module::<Game3dExample>()
		.name("Game 3D")
		.run()
}
