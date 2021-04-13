use newport::*;
use engine::{ Engine, EngineBuilder, Module };
use editor::Editor;

struct Gui;

impl Module for Gui {
    fn new() -> Self {
        Self
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Editor>()
    }
}

fn main() {
    let builder = EngineBuilder::new()
        .module::<Gui>()
        .name("Editor Example");
    Engine::run(builder);
}