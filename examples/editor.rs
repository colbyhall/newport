use newport::{
	editor::Editor,
	engine::{
		Builder,
		Engine,
		Module,
	},
};

// First thing first is to define our module struct
struct EditorExample;

// Implement the module trait
impl Module for EditorExample {
	fn new() -> Self {
		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Editor>()
	}
}

// Start the app runner
fn main() -> Result<(), std::io::Error> {
	Engine::builder()
		.module::<EditorExample>()
		.name("Editor")
		.run()
}
