use engine::Engine;
use core::module::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Test {
    a: String,
    b: i32,
    c: f32,
    d: Vec<u32>,
}

fn main() {
    let builder = ModuleBuilder::new();
    Engine::run(builder).unwrap();
}
