use newport::engine::{ ModuleBuilder, Engine };
use newport::asset::AssetManager;

fn main() {
    let builder = ModuleBuilder::new()
        .module::<AssetManager>();
    Engine::run(builder).unwrap();
}