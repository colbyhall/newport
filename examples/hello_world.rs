use newport::engine::{ Module, Engine, EngineBuilder };

struct HelloWorld;

impl Module for HelloWorld {
    fn new() -> Self {
        Self
    }
}

fn main() {
    let builder = EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World");
    Engine::run(builder);
}