use crate::{
    engine::{ Module, EngineBuilder, Engine },
    ecs::Entity,
};

use newport_editor::{
    Editor,
    View,
    AssetBrowser,
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
        let mut details_view = View::new("details_view", 0.2);
        details_view.add_tab(Details::new());

        let mut asset_browser = View::new("asset_browser", 0.4);
        asset_browser.add_tab(AssetBrowser::new());

        let mut level = View::new("level", 0.8);
        level.add_tab(Viewport::new());

        let mut level_outline = View::new("level_outline", 0.2);
        level_outline.add_tab(Outliner::new());

        let level_and_outliner = View::new_views("level_and_outliner", 0.6, vec![level, level_outline], Direction::RightToLeft);

        let level_and_asset_browser = View::new_views("scene_and_asset_browser", 0.8, vec![asset_browser, level_and_outliner], Direction::DownToUp);
        
        let view = View::new_views("main", 1.0, vec![details_view, level_and_asset_browser], Direction::RightToLeft);

        let editor = Engine::as_ref().module::<Editor>().unwrap();
        editor.set_view(view);

        Self(Mutex::new(GameEditorInner{
            selected_entity: None,
        }))
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Editor>()
    }
}