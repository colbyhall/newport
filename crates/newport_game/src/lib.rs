pub(crate) use newport_engine as engine;
pub(crate) use newport_ecs as ecs;
pub(crate) use newport_editor as editor;

use engine::{ Module, EngineBuilder };
use ecs::World;
use editor::Editor;

use std::sync::Mutex;

mod level_editor;
use level_editor::*;

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
