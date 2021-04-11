use newport::engine::*;

struct HelloWorld;

impl ModuleCompileTime for HelloWorld {
    fn new() -> Self {
        Self
    }
}

impl ModuleRuntime for HelloWorld {
}

fn main() {
    let builder = EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World".to_string());
    Engine::run(builder).unwrap();
}