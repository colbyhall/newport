use newport::engine::{ Module, Builder };

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
    Builder::new()
        .module::<HelloWorld>()
        .name("Hello World")
        .run()
}