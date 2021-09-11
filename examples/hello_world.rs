use newport::engine::{
	Engine,
	Module,
};

// First thing first is to define our module struct
struct HelloWorld;

// Implement the module trait
impl Module for HelloWorld {
	fn new() -> Self {
		Self
	}
}

// Start the app runner
fn main() -> Result<(), std::io::Error> {
	Engine::builder()
		.module::<HelloWorld>()
		.name("Hello World")
		.run()
}
