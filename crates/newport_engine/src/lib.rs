#![feature(trait_alias)]

use newport_core::containers::{ Box, HashMap };
use newport_os::window::{ WindowBuilder, Window, WindowEvent };

use std::any::{ TypeId, Any };
use std::sync::atomic::{ AtomicBool, Ordering };

static mut ENGINE: Option<Engine> = None;

// TODO: Document
pub struct Engine {
    name:    String,
    modules: HashMap<TypeId, Box<dyn ModuleRuntime>>, 

    is_running: AtomicBool,
    window:     Window,
}

impl Engine {
    pub fn run(mut builder: ModuleBuilder) -> Result<&'static Engine, ()> {      
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
                module.post_init(ENGINE.as_mut().unwrap());
            }
            builder.post_inits.drain(..).for_each(|init| init(ENGINE.as_mut().unwrap()));

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

    pub fn module<T: Module>(&'static self) -> Option<&'static T> {
        let id = TypeId::of::<T>();
        
        let module = self.modules.get(&id);
        if module.is_none() { return None; }
        let module = module.unwrap();
        module.as_any().downcast_ref::<T>()
    }

    pub fn module_mut<T: Module>(&'static mut self) -> Option<&'static mut T> {
        let id = TypeId::of::<T>();
        
        let module = self.modules.get_mut(&id);
        if module.is_none() { return None; }
        let module = module.unwrap();
        
        // UNSAFE: I'm lazy and I don't want to implement all of these manually 
        unsafe{
            let module = module.as_any() as *const dyn Any;
            let module = module as *mut dyn Any;
            let module = &mut *module;
            module.downcast_mut::<T>()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

struct ModuleBuilderEntry {
    id:     TypeId,
    spawn:  fn() -> Result<Box<dyn ModuleRuntime>, String>,
}

pub struct ModuleBuilder {
    entries:    Vec<ModuleBuilderEntry>,
    name:       Option<String>,
    post_inits: Vec<Box<dyn FnOnce(&'static mut Engine) + 'static>>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self { 
            entries:    Vec::with_capacity(32),
            name:       None,
            post_inits: Vec::new(),
        }
    }

    pub fn module<T: Module>(mut self) -> Self {
        fn spawn<T: Module>() -> Result<Box<dyn ModuleRuntime>, String> {
            let t = T::new()?;
            Ok(Box::new(t))
        }
        
        // Add dependencies to the entries list. There will be duplicates
        self = T::depends_on(self);

        // Push entry with generic spawn func and type id
        self.entries.push(ModuleBuilderEntry{
            id:     TypeId::of::<T>(),
            spawn:  spawn::<T>,
        });

        self
    }

    pub fn post_init<F: FnOnce(&'static mut Engine) + 'static>(mut self, f: F) -> Self {
        self.post_inits.push(Box::new(f));
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

pub trait ModuleCompileTime: Sized + 'static {
    fn new() -> Result<Self, String>;

    fn depends_on(builder: ModuleBuilder) -> ModuleBuilder {
        builder
    }
}

pub trait ModuleRuntime: Any {
    fn post_init(&'static mut self, _: &'static mut Engine) { }

    fn as_any(&self) -> &dyn Any;
}

pub trait Module = ModuleRuntime + ModuleCompileTime;