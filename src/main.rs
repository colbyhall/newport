use os::window::{ WindowBuilder, WindowEvent };
use asset::{ VariantRegistry, CollectionRegistry, AssetManager, AssetRef };
use log::*;
use core::containers::HashSet;

use std::env::{ current_exe, set_current_dir };
use std::path::PathBuf;
use std::fs::read_to_string;

#[derive(Debug)]
struct Test {
    data: String,
}

fn main() {
    // Ensure we're in the work directory
    {
        let mut new_path = current_exe().unwrap();
        new_path.pop();
        new_path.pop();
        new_path.pop();
        set_current_dir(new_path).unwrap();
    }

    Logger::init();

    log!("Hello, world!");

    let collections = CollectionRegistry::new()
        .add(PathBuf::from("assets/"));
        
    let mut variants = VariantRegistry::new();

    let mut exts = HashSet::new();
    exts.insert("test".to_string());

    variants.add(exts, |path| {
        let data = read_to_string(path).unwrap();
        println!("{:?}, {}", path, data);

        Ok(Test { data: data })
    }, |_| { });

    AssetManager::init(variants, collections);

    let test: AssetRef<Test> = AssetRef::new("assets/test.test").unwrap();
    {
        let read_lock = test.read();
        let test = &*read_lock;
        println!("{}", test.data);
    }

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
