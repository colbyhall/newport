use crate::{
    engine::{ Module, Builder, Engine },
    ecs::Entity,
};

use newport_editor::{
    Editor,
    View,
    Direction,
};

mod outliner;
mod viewport;
mod details;

pub use {
    outliner::*,
    viewport::*,
    details::*,
};

use std::sync::Mutex;

pub struct GameEditorInner {
    pub selected_entity: Option<Entity>,
}

pub struct GameEditor(Mutex<GameEditorInner>);

impl Module for GameEditor {
    fn new() -> Self {
        let mut details_view = View::new("details_view", 0.7);
        details_view.add_tab(Details::new());

        let mut outliner_view = View::new("outliner_view", 0.3);
        outliner_view.add_tab(Outliner::new());

        let details_and_outliner = View::new_views("details_and_outliner", 0.2, vec![details_view, outliner_view], Direction::DownToUp);

        let mut level = View::new("level", 0.8);
        level.add_tab(Viewport::new());
        level.hide_tabs(true);
        level.hide_border(true);

        let view = View::new_views("main", 1.0, vec![details_and_outliner, level], Direction::RightToLeft);

        let editor = Engine::as_ref().module::<Editor>().unwrap();
        editor.set_view(view);

        Self(Mutex::new(GameEditorInner{
            selected_entity: None,
        }))
    }

    fn depends_on(builder: Builder) -> Builder {
        builder
            .module::<Editor>()
    }
}