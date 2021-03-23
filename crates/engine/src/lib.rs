use core::containers::{ Box, HashMap };
use std::any::{ Any, TypeId };

struct ModuleEntry {
    module:  Box<dyn Any>, // Stored as dyn Any for downcast
}

pub mod module;

// TODO: Document
pub struct Engine {
    modules: HashMap<TypeId, ModuleEntry>, 
}

static mut ENGINE: Option<Engine> = None;

impl Engine {
    /// Returns the global [`Engine`] as a ref
    pub fn as_ref() -> &'static Engine {
        unsafe{ ENGINE.as_ref().unwrap() }
    }

    pub fn find_module<T: Module>(&'static self) -> Option<&'static T> {
        let id = TypeId::of::<T>();
        
        let entry = self.modules.get(&id);
        if entry.is_none() { return None; }
        let entry = entry.unwrap();
        entry.module.as_ref().downcast_ref::<T>()
    }
}
