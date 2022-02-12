use engine::{
	define_log_category,
	error,
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

static CACHE_PATH: &str = "target/cache/";

define_log_category!(Cache, CACHE_CATEGORY);

pub struct CacheManager {
	caches: HashMap<TypeId, Box<dyn Any>>,
}

pub struct CacheRef<T: Cache> {
	cache: &'static Box<dyn Any>,
	phantom: PhantomData<T>,
}

impl<T: Cache> CacheRef<T> {
	pub fn new() -> Option<Self> {
		let manager: &CacheManager = Engine::module()?;

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
		let path = Path::new(CACHE_PATH);
		if !path.exists() {
			fs::create_dir(path).unwrap();
		}

		let registers: HashMap<TypeId, CacheVariant> = Engine::register::<CacheVariant>()
			.iter()
			.map(|f| (f.id, f.clone()))
			.collect();

		let mut caches = HashMap::with_capacity(registers.len());
		for (id, register) in registers.iter() {
			let (save, cache) = if register.path.exists() {
				let file = fs::read(&register.path).unwrap();
				match (register.deserialize)(file) {
					Ok(mut cache) => {
						let save = (register.reload)(&mut cache);
						(save, cache)
					}
					Err(err) => {
						error!(CACHE_CATEGORY, "{}", err);
						(true, (register.new)())
					}
				}
			} else {
				(true, (register.new)())
			};

			if save {
				let contents = (register.serialize)(&cache);
				fs::write(&register.path, contents).unwrap();
			}

			caches.insert(*id, cache);
		}

		Self { caches }
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder.tick(|_| {
			// TODO: Reload
		})
	}
}

#[derive(Clone)]
pub struct CacheVariant {
	path: PathBuf,
	id: TypeId,

	serialize: fn(&Box<dyn Any>) -> Vec<u8>,
	deserialize: fn(Vec<u8>) -> bincode::Result<Box<dyn Any>>,
	new: fn() -> Box<dyn Any>,
	reload: fn(&mut Box<dyn Any>) -> bool,
}

pub trait Cache: Serialize + DeserializeOwned + 'static {
	fn new() -> Self;
	fn reload(&mut self) -> bool;

	fn variant() -> CacheVariant {
		fn serialize<T: Cache>(cache: &Box<dyn Any>) -> Vec<u8> {
			let t = cache.downcast_ref::<T>().unwrap();
			bincode::serialize(t).unwrap()
		}

		fn deserialize<T: Cache>(data: Vec<u8>) -> bincode::Result<Box<dyn Any>> {
			let t: T = bincode::deserialize(&data)?;
			Ok(Box::new(t))
		}

		fn new<T: Cache>() -> Box<dyn Any> {
			let t = T::new();
			Box::new(t)
		}

		fn reload<T: Cache>(cache: &mut Box<dyn Any>) -> bool {
			let t = cache.downcast_mut::<T>().unwrap();
			t.reload()
		}

		let type_name = std::any::type_name::<Self>();
		let name = type_name
			.rsplit_once("::")
			.unwrap_or(("", type_name))
			.1
			.to_lowercase();
		let name = name.rsplit_once("cache").unwrap_or((&name, "")).0;

		let mut path = PathBuf::from(CACHE_PATH);
		let file_name = format!("{}.bin", name);
		path.push(file_name);

		CacheVariant {
			path,
			id: TypeId::of::<Self>(),

			serialize: serialize::<Self>,
			deserialize: deserialize::<Self>,
			new: new::<Self>,
			reload: reload::<Self>,
		}
	}
}
