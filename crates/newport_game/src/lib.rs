use newport_engine::{ Module, EngineBuilder };
use newport_ecs::World;
use newport_editor::Editor;

use std::sync::Mutex;

pub struct Game {
    pub world: Mutex<World>,
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
    }

}
