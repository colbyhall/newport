use newport_ecs::Entity;

use crate::Game;

#[derive(Default)]
pub struct WorldPage {
    selected: Option<Entity>,
}
