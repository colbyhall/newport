use newport::engine::*;
use newport::asset::{ AssetManager, AssetRef, from_str, Path, PathBuf, Asset, LoadError };
use newport::core::containers::HashSet;
use newport::log::*;
use newport::gpu::SelectedGPU;

use std::fs::read_to_string;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Test {
    a: String,
    b: i32,
    c: f32,
    d: Vec<i32>
}

struct HelloWorld;

impl ModuleCompileTime for HelloWorld {
    fn new() -> Result<Self, String> {
        Ok(HelloWorld)
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder.module::<SelectedGPU>()
    }
}

impl ModuleRuntime for HelloWorld {
    fn post_init(&mut self, engine: &mut Engine) {
        let asset_manager = engine.module_mut::<AssetManager>().unwrap();

        let mut exts = HashSet::new();
        exts.insert("test".to_string());

        fn load_test(path: &Path) -> Result<Box<dyn Asset>, LoadError> {
            let result: Test = from_str(&read_to_string(path).unwrap()).unwrap();
            Ok(Box::new(result))
        }

        fn unload_test(_: Box<dyn Asset>) { }

        asset_manager
            .register_collection(PathBuf::from("assets/"))
            .register_variant::<Test>(exts, load_test, unload_test);
    }

    fn on_startup(&mut self) {
        let engine = Engine::as_ref();

        let asset_manager = engine.module::<AssetManager>().unwrap();
        let test: AssetRef<Test> = asset_manager.find("assets/test.test").unwrap();
        info!("[HelloWorld] {:?}", test);
    }
}

fn main() {
    let builder = EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World".to_string());
    Engine::run(builder).unwrap();
}