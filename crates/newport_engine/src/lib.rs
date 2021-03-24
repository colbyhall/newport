#![feature(trait_alias)]

use newport_core::containers::{ Box, HashMap };
use newport_os::window::{ WindowBuilder, Window, WindowEvent };

use std::any::TypeId;
use std::sync::atomic::{ AtomicBool, Ordering };

pub use std::any::Any;

static mut ENGINE: Option<Engine> = None;

/// Global runnable structure used for instantiating engine modules and handling app code
/// 
/// Created using an [`EngineBuilder`] which defines the functionality of the app using [`Module`]s 
pub struct Engine {
    name:    String,
    modules: HashMap<TypeId, Box<dyn ModuleRuntime>>, 

    is_running: AtomicBool,
    window:     Window,
}

impl Engine {
    /// Starts the engine using what was built with a [`EngineBuilder`]
    /// 
    /// # Arguments
    /// 
    /// * `builder` - An [`EngineBuilder`] used to setup app execution and structure
    /// 
    /// # Examples
    /// 
    /// ```
    /// use newport_engine::{ EngineBuilder, Engine };
    /// use newport_asset::AssetManager;
    /// 
    /// let builder = EngineBuilder::new()
    ///     .module::<AssetManager>();
    /// Engine::run(builder).unwrap();
    /// ```
    pub fn run(mut builder: EngineBuilder) -> Result<&'static Engine, ()> {      
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

            // Do on_startup
            engine.modules.iter_mut().for_each(|(_, module)| module.on_startup());

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

    /// Searches a module by type and returns an [`Option<&'static T>`]
    /// 
    /// # Arguments 
    /// 
    /// * `T` - A [`Module`] that should have been created using a [`EngineBuilder`]
    /// 
    /// # Examples 
    /// 
    /// ```
    /// use newport_engine::Engine;
    /// 
    /// let engine = Engine::as_ref();
    /// let module = engine.module::<Module>().unwrap();
    /// ```
    pub fn module<T: Module>(&'static self) -> Option<&'static T> {
        let id = TypeId::of::<T>();
        
        let module = self.modules.get(&id);
        if module.is_none() { return None; }
        let module = module.unwrap();
        module.as_any().downcast_ref::<T>()
    }

    /// Searches a module by type and returns an [`Option<&'static mut T>`]
    /// 
    /// # Arguments 
    /// 
    /// * `T` - A [`Module`] that should have been created using a [`EngineBuilder`]
    /// 
    /// # Examples 
    /// 
    /// ```
    /// use newport_engine::Engine;
    /// 
    /// let engine = Engine::as_ref();
    /// let module = engine.module_mut::<Module>().unwrap();
    /// ```
    pub fn module_mut<T: Module>(&mut self) -> Option<&mut T> {
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

    /// Returns the name of the engine runnable
    pub fn name(&self) -> &str {
        &self.name
    }
}

struct EngineBuilderEntry {
    id:     TypeId,
    spawn:  fn() -> Result<Box<dyn ModuleRuntime>, String>,
}

/// Structure used to define engine structure and execution
pub struct EngineBuilder {
    entries:    Vec<EngineBuilderEntry>,
    name:       Option<String>,
    post_inits: Vec<Box<dyn FnOnce(&'static mut Engine) + 'static>>,
}

impl EngineBuilder {
    /// Creates a new [`EngineBuilder`]
    pub fn new() -> Self {
        Self { 
            entries:    Vec::with_capacity(32),
            name:       None,
            post_inits: Vec::new(),
        }
    }

    /// Adds a module to the list
    /// 
    /// # Arguments
    /// 
    /// * `T` - A [`Module`] that will be initialized and used at runtime
    /// 
    /// # Examples
    /// 
    /// ```
    /// use newport_engine::EngineBuilder;
    /// 
    /// let builder = EngineBuilder::new()
    ///     .module::<Test>();
    /// ```
    pub fn module<T: Module>(mut self) -> Self {
        fn spawn<T: Module>() -> Result<Box<dyn ModuleRuntime>, String> {
            let t = T::new()?;
            Ok(Box::new(t))
        }
        
        // Add dependencies to the entries list. There will be duplicates
        self = T::depends_on(self);

        // Push entry with generic spawn func and type id
        self.entries.push(EngineBuilderEntry{
            id:     TypeId::of::<T>(),
            spawn:  spawn::<T>,
        });

        self
    }

    /// Adds a post initialization closure to the list
    /// 
    /// # Arguments
    /// 
    /// * `T` - A [`Module`] that will be initialized and used at runtime
    /// 
    /// # Examples
    /// 
    /// ```
    /// use newport_engine::EngineBuilder;
    /// 
    /// let builder = EngineBuilder::new()
    ///     .module::<Test>();
    /// ```
    pub fn post_init<F: FnOnce(&'static mut Engine) + 'static>(mut self, f: F) -> Self {
        self.post_inits.push(Box::new(f));
        self
    }

    /// Sets the name of the engine runnable
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

/// Compile time element of [`Module`]s. Required to be split for dyn
pub trait ModuleCompileTime: Sized + 'static {
    /// Creates a module and returns as result. This is the initialization point for Modules
    /// 
    /// # Notes
    /// 
    /// * [`Engine`] is not available during this function
    fn new() -> Result<Self, String>;

    /// Takes a builder to append on other modules or elements
    /// 
    /// # Arguments
    /// 
    /// * `builder` - A [`EngineBuilder`] used to add dep modules or functions
    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
    }
}

/// Runtime element of [`Module`]s
pub trait ModuleRuntime: Any {
    /// Called after all modules are initialized
    fn post_init(&mut self, _: &mut Engine) { }

    // Called after post initialization but before main loop
    fn on_startup(&'static mut self) { }

    fn as_any(&self) -> &dyn Any;
}

/// Combined Module trait
pub trait Module = ModuleRuntime + ModuleCompileTime;