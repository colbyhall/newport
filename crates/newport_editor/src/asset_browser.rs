use crate::{
    Tab,
    Builder,

    Layout,
    Scrollbox,
    Direction,

    engine::Engine,
    asset::AssetManager,
};

use std::{
    path::PathBuf,
};

enum BrowserEntry {
    Directory{
        path:    PathBuf,
        entries: Vec<BrowserEntry>
    },
    Asset(PathBuf),
}

pub struct AssetBrowser {
    entries: Vec<BrowserEntry>,
}

impl AssetBrowser {
    pub fn new() -> Self {
        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();

        let assets = asset_manager.assets();
        for entry in assets.iter() {

        }

        Self {
            entries: Vec::new(), // TEMP
        }
    }
}

impl Tab for AssetBrowser {
    fn name(&self) -> String {
        "Asset Browser".to_string()
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.layout(Layout::left_to_right(builder.layout.bounds()), |builder| {
            let bounds = builder.layout.bounds();

            let size = (300.0, bounds.height()).into();
            Scrollbox::new("test", builder.layout.push_size(size), Direction::UpToDown)
                .build(builder, |builder| {
                    for i in 0..120 {
                        builder.label(format!("Test {}", i));
                    }
                });
        });
    }
}