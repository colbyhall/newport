use newport_engine as engine;
use newport_serde as serde;

use engine::{
    Module,
    Engine,
    EngineBuilder
};

use serde::{
    Serialize,
    de::DeserializeOwned,

    bincode,
};

use std::{
    any::{
        TypeId,
        Any,
    },
    collections::HashMap,
    path::{ PathBuf, Path },
    fs,
    sync::{ RwLock, RwLockReadGuard },
    marker::PhantomData,
    ops::Deref,
};

static CACHE_PATH: &'static str = "cache/";

pub struct CacheManager {
    registers: HashMap<TypeId, CacheRegister>,
    caches:    HashMap<TypeId, RwLock<Box<dyn Any>>>,
}

pub struct CacheViewer<'a, T: Cache> {
    phantom: PhantomData<T>,
    lock:    RwLockReadGuard<'a, Box<dyn Any>>,
}

impl<'a, T: Cache> Deref for CacheViewer<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.lock.downcast_ref().unwrap()
    }
}

impl CacheManager {
    pub fn cache<T: Cache>(&self) -> Option<CacheViewer<T>> {
        let id = TypeId::of::<T>();

        let cache = self.caches.get(&id)?;
        let lock = cache.read().ok()?;
        Some(CacheViewer{
            phantom: PhantomData,
            lock,
        })
    }
}

impl Module for CacheManager {
    fn new() -> Self {
        let engine = Engine::as_ref();

        let path = Path::new(CACHE_PATH);
        if !path.exists() {
            fs::create_dir(path).unwrap();
        }

        let mut cache_registers: Vec<CacheRegister> = engine.register().unwrap_or_default();
        let mut registers = HashMap::with_capacity(cache_registers.len());
        cache_registers.drain(..).for_each(|f| {
            registers.insert(f.id, f);
        });

        let mut caches = HashMap::with_capacity(registers.len());
        for (id, register) in registers.iter() {
            let path = register.path();

            let cache = if path.exists() {
                let file = fs::read(path).unwrap();
                (register.deserialize)(file)
            } else {
                let cache = (register.new)();

                let contents = (register.serialize)(&cache);
                fs::write(path, contents).unwrap();

                cache
            };

            caches.insert(*id, RwLock::new(cache));
        }

        Self {
            registers,
            caches
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .tick(|engine: &Engine, _: f32| {
                let cache_manager = engine.module::<CacheManager>().unwrap();

                for (id, cache) in cache_manager.caches.iter() {
                    let mut cache = cache.write().unwrap();

                    let register = cache_manager.registers.get(id).unwrap();
                    if (register.needs_reload)(&cache) {
                        *cache = (register.new)();

                        let contents = (register.serialize)(&cache);
                        fs::write(register.path(), contents).unwrap();
                    }
                }
            })
    }
}

impl Drop for CacheManager {
    fn drop(&mut self) {
        let Self{
            caches,
            registers
        } = self;

        caches.drain().for_each(|(id, cache)| {
            let register = registers.get(&id).unwrap();
            let path = register.path();

            let cache = cache.read().unwrap();
            let contents = (register.serialize)(&cache);

            fs::write(path, contents).unwrap();
        });
    }
}

#[derive(Clone)]
pub struct CacheRegister {
    name: &'static str,
    id:   TypeId,

    serialize:    fn(&Box<dyn Any>) -> Vec<u8>,
    deserialize:  fn(Vec<u8>) -> Box<dyn Any>,
    new:          fn() -> Box<dyn Any>,
    needs_reload: fn(&Box<dyn Any>) -> bool,
}

impl CacheRegister {
    fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(CACHE_PATH);
        let file_name = format!("{}.cache", self.name);
        path.push(file_name);
        path
    }
}

pub trait Cache: Serialize + DeserializeOwned + 'static {
    fn new() -> Self;
    fn needs_reload(&self) -> bool;
}

impl CacheRegister {
    pub fn new<T: Cache>(name: &'static str) -> Self {
        fn serialize<T: Cache>(cache: &Box<dyn Any>) -> Vec<u8> {
            let t = cache.downcast_ref::<T>().unwrap();
            bincode::serialize(t).unwrap()
        }

        fn deserialize<T: Cache>(data: Vec<u8>) -> Box<dyn Any> {
            let t: T = bincode::deserialize(&data).unwrap();
            Box::new(t)
        }

        fn new<T: Cache>() -> Box<dyn Any> {
            let t = T::new();
            Box::new(t)
        }

        fn needs_reload<T: Cache>(cache: &Box<dyn Any>) -> bool {
            let t = cache.downcast_ref::<T>().unwrap();
            t.needs_reload()
        }

        Self {
            name,
            id: TypeId::of::<T>(),

            serialize: serialize::<T>,
            deserialize: deserialize::<T>,
            new: new::<T>,
            needs_reload: needs_reload::<T>,
        }
    }
}