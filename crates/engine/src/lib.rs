use core::containers::{ Box, HashMap };
use core::module::*;
use os::window::{ WindowBuilder, Window, WindowEvent };
use asset::AssetManager;
use log::*;

use std::any::TypeId;
use std::sync::atomic::{ AtomicBool, Ordering };

// TODO: Document
pub struct Engine {
    name:    String,
    modules: HashMap<TypeId, Box<dyn ModuleRuntime>>, 

    is_running: AtomicBool,
    window:     Window,
}

static mut ENGINE: Option<Engine> = None;

impl Engine {
    pub fn run(mut builder: ModuleBuilder) -> Result<&'static Engine, ()> {
        // Default modules added here
        builder = builder
            .module::<Logger>()
            .module::<AssetManager>();
        
        // NOTE: ModuleCompileTime::new() happens before engine is initialized
        let mut modules = HashMap::with_capacity(builder.entries.len());
        for it in builder.entries {
            if modules.contains_key(&it.id) { continue; }
            modules.insert(it.id, (it.spawn)().unwrap());
        }

        // Grab the project name or use a default
        let name;
        if builder.name.is_some() {
            name = builder.name.unwrap();
        } else {
            name = "project".to_string();
        }
    
        // UNSAFE: Set the global state
        unsafe{ 
            let window = WindowBuilder::new()
                .title(name.clone())
                .spawn()
                .unwrap();

            ENGINE = Some(Engine{
                name:       name,
                modules:    modules,
                is_running: AtomicBool::new(true),
                window:     window,
            });

            let engine = ENGINE.as_mut().unwrap();

            // Do post init
            for (_, module) in engine.modules.iter_mut() {
                module.post_init();
            }

            // let asset_manager = engine.find_module::<AssetManager>();
            // {
            // }

            engine.window.set_visible(true);

            // Game loop
            'run: while engine.is_running.load(Ordering::Relaxed) {
                for event in engine.window.poll_events() {
                    match event {
                        WindowEvent::Closed => {
                            engine.is_running.store(false, Ordering::Relaxed);
                            break 'run;
                        }
                        _ => { }
                    }
                }
            }
        }

        Ok(Self::as_ref())
    }

    /// Returns the global [`Engine`] as a ref
    pub fn as_ref() -> &'static Engine {
        unsafe{ ENGINE.as_ref().unwrap() }
    }

    pub fn find_module<T: Module>(&'static self) -> Option<&'static T> {
        let id = TypeId::of::<T>();
        
        let module = self.modules.get(&id);
        if module.is_none() { return None; }
        let module = module.unwrap();
        module.as_any().downcast_ref::<T>()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod test;