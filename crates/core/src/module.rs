use std::any::{ TypeId, Any };

pub struct ModuleBuilderEntry {
    pub id:     TypeId,
    pub spawn:  fn() -> Result<Box<dyn ModuleRuntime>, String>,
}

pub struct ModuleBuilder {
    pub entries: Vec<ModuleBuilderEntry>,
    pub name:    Option<String>,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self { 
            entries: Vec::with_capacity(32),
            name:   None,
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
    fn post_init(&'static mut self) { }

    fn as_any(&self) -> &dyn Any;
}

pub trait Module = ModuleRuntime + ModuleCompileTime;