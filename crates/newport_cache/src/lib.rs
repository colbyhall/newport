use newport_engine as engine;
use newport_serde as serde;

use engine::{
    Module,
};

use serde::{
    Serialize,
    de::DeserializeOwned,

    bincode,
};

pub struct CacheManager {

}

impl Module for CacheManager {
    fn new() -> Self {
        Self {

        }
    }
}

pub struct CacheRegister {
    name: &'static str,

    serialize:    fn(Box<dyn Any>) -> Vec<u8>,
    deserialize:  fn(Vec<u8>) -> Box<dyn Any>,
    new:          fn() -> Box<dyn Any>,
    needs_reload: fn(&Box<dyn Any>) -> bool,
}

use std::any::Any;

pub trait Cache: Serialize + DeserializeOwned + 'static {
    fn new() -> Self;
    fn needs_reload(&self) -> bool;
}

impl CacheRegister {
    pub fn new<T: Cache>(name: &'static str) -> Self {
        fn serialize<T: Cache>(cache: Box<dyn Any>) -> Vec<u8> {
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

            serialize: serialize::<T>,
            deserialize: deserialize::<T>,
            new: new::<T>,
            needs_reload: needs_reload::<T>,
        }
    }
}