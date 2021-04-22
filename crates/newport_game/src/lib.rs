use newport_engine::{ Module, EngineBuilder, Engine };
use newport_ecs::World;
use newport_editor::Editor;

use std::sync::Mutex;

mod editor;

pub struct Game {
    world: Mutex<World>,
}

impl Module for Game {
    fn new() -> Self {
        Self{ 
            world: Mutex::new(World::new())
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Editor>()
            .post_init(|engine: &Engine| {
                let editor = engine.module::<Editor>().unwrap();
                // editor.push_page(Box::new(editor::WorldPage::default()));
            })
    }

}
