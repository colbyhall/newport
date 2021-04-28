#![feature(trait_alias)]

use newport_core::containers::{ Box, HashMap };
use newport_os::window::{ WindowBuilder, WindowStyle };
pub use newport_os::window::{ WindowEvent, Window };
use newport_math::Rect;

use std::any::TypeId;
use std::sync::{ Mutex };
use std::sync::atomic::{ AtomicBool, Ordering };
use std::any::Any;
use std::time::Instant;
use std::convert::Into;

static mut ENGINE: Option<Engine> = None;

/// Global runnable structure used for instantiating engine modules and handling app code
/// 
/// Created using an [`EngineBuilder`] which defines the functionality of the app using [`Module`]s 
pub struct Engine {
    name:    String,
    modules: HashMap<TypeId, Box<dyn Any>>, 

    is_running: AtomicBool,
    fps:        i32,
    
    window:   Window,
    minimize: AtomicBool,
    maximize: AtomicBool,
    drag:     Mutex<Rect>,
    dpi:      f32,
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
    pub fn run(mut builder: EngineBuilder) {      
        // Grab the project name or use a default
        let name = builder.name.unwrap_or("newport".to_string());
    
        // UNSAFE: Set the global state
        let engine = unsafe{ 
            let window = WindowBuilder::new()
                .title(name.clone())
                .style(WindowStyle::CustomTitleBar{
                    border: 3.0,
                    drag:   Default::default(),
                })
                .spawn()
                .unwrap();

            let dpi = window.dpi();

            ENGINE = Some(Engine{
                name:       name,
                modules:    HashMap::with_capacity(builder.entries.len()),
                is_running: AtomicBool::new(true),
                fps:        0,
                
                window:   window,
                minimize: AtomicBool::new(false),
                maximize: AtomicBool::new(false),

                drag: Mutex::new(Rect::default()),
                dpi:  dpi,
            });

            ENGINE.as_mut().unwrap()
        };

        // NOTE: All modules a module depends on will be available at initialization
        builder.entries.drain(..).for_each(|it| {
            engine.modules.insert(it.id, (it.spawn)());
        });

        // Do post init
        builder.post_inits.drain(..).for_each(|init| init(engine));

        engine.window.set_visible(true);
        engine.window.maximize();

        let mut frame_count = 0;
        let mut time = 0.0;

        // Game loop
        let mut last_frame_time = Instant::now();
        'run: while engine.is_running.load(Ordering::Relaxed) {
            let now = Instant::now();
            let dt = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            time += dt;
            if time >= 1.0 {
                time = 0.0;
                engine.fps = frame_count;
                frame_count = 0;
            }
            frame_count += 1;

            {    
                for event in engine.window.poll_events() {
                    builder.process_input.iter().for_each(|process_input| process_input(engine, &engine.window, &event));
    
                    match event {
                        WindowEvent::Closed => {
                            engine.is_running.store(false, Ordering::Relaxed);
                            break 'run;
                        },
                        WindowEvent::Resizing(_, _) => {
                            builder.tick.iter().for_each(|tick| tick(engine, 0.0));
                        }
                        _ => {}
                    }
                }
            }

            builder.tick.iter().for_each(|tick| tick(engine, dt));

            if engine.maximize.load(Ordering::Relaxed) {
                engine.window.maximize();
                engine.maximize.store(false, Ordering::Relaxed)
            }

            if engine.minimize.load(Ordering::Relaxed) {
                engine.window.minimize();
                engine.minimize.store(false, Ordering::Relaxed)
            }

            {
                let drag = engine.drag.lock().unwrap();
                engine.window.set_custom_drag(*drag);
            }
        }

        // Do pre shutdowns
        builder.pre_shutdown.drain(..).for_each(|shutdown| shutdown(engine));
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
    pub fn module<'a, T: Module>(&'a self) -> Option<&'a T> {
        let id = TypeId::of::<T>();
        
        let module = self.modules.get(&id)?;
        module.downcast_ref::<T>()
    }

    /// Returns the name of the engine runnable
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the window that the engine draws into
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn shutdown(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }

    pub fn fps(&self) -> i32 {
        self.fps
    }

    pub fn maximize(&self) {
        self.maximize.store(true, Ordering::Relaxed);
    }

    pub fn minimize(&self) {
        self.minimize.store(true, Ordering::Relaxed);
    }

    pub fn set_custom_drag(&self, drag: Rect) {
        let mut self_drag = self.drag.lock().unwrap();
        *self_drag = drag;
    }

    pub fn dpi(&self) -> f32 {
        self.dpi
    }
}

struct EngineBuilderEntry {
    id:     TypeId,
    spawn:  fn() -> Box<dyn Any>,
}

pub trait PostInit      = FnOnce(&Engine) + 'static;
pub trait ProcessInput  = Fn(&Engine, &Window, &WindowEvent) + 'static;
pub trait Tick          = Fn(&Engine, f32) + 'static;
pub trait PreShutdown   = FnOnce(&Engine) + 'static;

/// Structure used to define engine structure and execution
pub struct EngineBuilder {
    entries:    Vec<EngineBuilderEntry>,
    name:       Option<String>,

    post_inits:     Vec<Box<dyn PostInit>>,
    process_input:  Vec<Box<dyn ProcessInput>>,
    tick:           Vec<Box<dyn Tick>>,
    pre_shutdown:   Vec<Box<dyn PreShutdown>>,
}

impl EngineBuilder {
    /// Creates a new [`EngineBuilder`]
    pub fn new() -> Self {
        Self { 
            entries:    Vec::with_capacity(32),
            name:       None,

            post_inits:     Vec::new(),
            process_input:  Vec::new(),
            tick:           Vec::new(),
            pre_shutdown:   Vec::new(),
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
        // Don't add another module thats already added
        let id = TypeId::of::<T>();
        for it in self.entries.iter() {
            if it.id == id {
                return self;
            }
        }

        fn spawn<T: Module>() -> Box<dyn Any> {
            Box::new(T::new())
        }
        
        // Add dependencies to the entries list. There will be duplicates
        self = T::depends_on(self);

        // Push entry with generic spawn func and type id
        self.entries.push(EngineBuilderEntry{
            id:     id,
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
    pub fn post_init<F: PostInit>(mut self, f: F) -> Self {
        self.post_inits.push(Box::new(f));
        self
    }

    /// Adds a post initialization closure to the list
    pub fn process_input<F: ProcessInput>(mut self, f: F) -> Self {
        self.process_input.push(Box::new(f));
        self
    }

    /// Adds a tick closure to the list
    pub fn tick<F: Tick>(mut self, f: F) -> Self {
        self.tick.push(Box::new(f));
        self
    }

    /// Adds a pre shutdown closure to the list
    pub fn pre_shutdown<F: PreShutdown>(mut self, f: F) -> Self {
        self.pre_shutdown.push(Box::new(f));
        self
    }

    /// Sets the name of the engine runnable
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Modules are an easy way to have global immutable state
pub trait Module: Sized + 'static {
    /// Creates a module and returns as result. This is the initialization point for Modules
    fn new() -> Self;

    /// Takes a builder to append on other modules or elements
    /// 
    /// # Arguments
    /// 
    /// * `builder` - A [`EngineBuilder`] used to add dep modules or functions
    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
    }
}