use os::window::{ WindowBuilder, WindowEvent };
use asset::{ VariantRegistry, CollectionRegistry, AssetManager, AssetRef };
use log::*;
use core::containers::HashSet;

use std::path::PathBuf;
use std::fs::read_to_string;

#[derive(Debug)]
struct Test {
    data: String,
}

fn main() {
    Logger::init();

    let mut collections = CollectionRegistry::new();
    collections
        .add(PathBuf::from("assets/"));
        
    let mut exts = HashSet::new();
    exts.insert("test".to_string());

    let mut variants = VariantRegistry::new();
    variants
        .add(exts, |path| Ok(Test { data: read_to_string(path).unwrap() }), |_| { });

    AssetManager::init(variants, collections);

    let test: AssetRef<Test> = AssetRef::new("assets/test.test").unwrap();
    println!("{:?}", test);

    let mut window = WindowBuilder::new()
        .title("Hello, world!".to_string())
        .size((1280, 720))
        .spawn()
        .unwrap();
    
    window.set_visible(true);

    'run: loop {
        for event in window.poll_events() {
            match event {
                WindowEvent::Closed => {
                    break 'run;
                },
                _ => { }
            }  
        }
    }
}
