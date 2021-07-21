use newport_engine as engine;
use newport_serde as serde;

use engine::{
	Builder,
	Engine,
	Module,
};

use serde::{
	bincode,
	de::DeserializeOwned,

	Serialize,
};

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
	fs,
	marker::PhantomData,
	ops::Deref,
	path::{
		Path,
		PathBuf,
	},
};

static CACHE_PATH: &'static str = "cache/";

pub struct CacheManager {
	registers: HashMap<TypeId, CacheRegister>,
	caches: HashMap<TypeId, Box<dyn Any>>,
}

pub struct CacheRef<T: Cache> {
	cache: &'static Box<dyn Any>,
	phantom: PhantomData<T>,
}

impl<T: Cache> CacheRef<T> {
	pub fn new() -> Option<Self> {
		let engine = Engine::as_ref();
		let manager: &CacheManager = engine.module()?;

		let cache = manager.caches.get(&TypeId::of::<T>())?;

		Some(Self {
			cache,
			phantom: PhantomData,
		})
	}
}

impl<T: Cache> Deref for CacheRef<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		self.cache.downcast_ref::<T>().unwrap()
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

			caches.insert(*id, cache);
		}

		Self { registers, caches }
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.tick(|_engine: &Engine, _: f32| {
			// TODO: Reload
		})
	}
}

impl Drop for CacheManager {
	fn drop(&mut self) {
		let Self { caches, registers } = self;

		caches.drain().for_each(|(id, cache)| {
			let register = registers.get(&id).unwrap();
			let path = register.path();

			let contents = (register.serialize)(&cache);

			fs::write(path, contents).unwrap();
		});
	}
}

#[derive(Clone)]
pub struct CacheRegister {
	name: &'static str,
	id: TypeId,

	serialize: fn(&Box<dyn Any>) -> Vec<u8>,
	deserialize: fn(Vec<u8>) -> Box<dyn Any>,
	new: fn() -> Box<dyn Any>,
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
