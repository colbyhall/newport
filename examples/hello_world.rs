use newport::*;

use std::fs::read_to_string;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Test {
    a: String,
    b: i32,
    c: f32,
    d: Vec<i32>
}

impl asset::Asset for Test {
    fn load(path: &asset::Path) -> Result<Self, asset::LoadError> {
        let t = asset::from_str(&read_to_string(path).unwrap()).unwrap();
        Ok(t)
    }

    fn unload(_asset: Self) { }

    fn extension() -> &'static str { "test" }
}

struct HelloWorld;

impl engine::ModuleCompileTime for HelloWorld {
    fn new() -> Self {
        HelloWorld
    }

    fn depends_on(builder: engine::EngineBuilder) -> engine::EngineBuilder {
        builder.module::<gpu::GPU>()
    }
}

impl engine::ModuleRuntime for HelloWorld {
    fn post_init(&mut self, engine: &mut engine::Engine) {
        let asset_manager = engine.module_mut::<asset::AssetManager>().unwrap();

        asset_manager
            .register_collection(asset::PathBuf::from("assets/"))
            .register_variant::<Test>();
    }

    fn on_startup(&mut self) {
        let engine = engine::Engine::as_ref();

        let asset_manager = engine.module::<asset::AssetManager>().unwrap();
        let test: asset::AssetRef<Test> = asset_manager.find("assets/test.test").unwrap();
        info!("[HelloWorld] {:?}", test);
    }
}

fn main() {
    let builder = engine::EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World".to_string());
    engine::Engine::run(builder).unwrap();
}