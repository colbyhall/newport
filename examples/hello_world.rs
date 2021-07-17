use newport::engine::{Engine, EngineBuilder, Module};

// First thing first is to define our module struct
struct HelloWorld;

// Implement the module trait
impl Module for HelloWorld {
    fn new() -> Self {
        Self
    }
}

// Start the app runner
fn main() {
    let builder = EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World");
    Engine::run(builder);
}
