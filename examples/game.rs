use newport::{
    engine,
    game,
};

use engine::{ Module, Builder };
use game::Game;

struct GameExample;

// Implement the module trait
impl Module for GameExample {
    fn new() -> Self {
        Self
    }

    fn depends_on(builder: Builder) -> Builder {
        builder
            .module::<Game>()
    }
}

// Start the app runner
fn main() {
    Builder::new()
        .module::<GameExample>()
        .name("Game Example")
        .run();
}