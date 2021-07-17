use crate::{
	cache,
	engine,
	log,
	Asset,
	AssetCache,
	AssetCollection,
	AssetRef,
	AssetVariant,
	UUID,
};

use engine::{
	Builder,
	Engine,
	Module,
};

use log::{
	info,
	Logger,
};

use cache::{
	CacheManager,
	CacheRegister,
};

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
	fs,
	marker::PhantomData,
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		RwLock,
		RwLockReadGuard,
	},
	time::Instant,
	time::SystemTime,
};

#[derive(Debug)]
pub struct AssetEntry {
	pub variant: usize, // Index into AssetManager::variants
	pub write_time: SystemTime,

	// Arc which is used for asset ref counting. As long as there is more
	// than 1 reference the asset is ensured to be loaded
	pub asset: Arc<RwLock<Option<Box<dyn Any>>>>,
}

pub struct AssetManagerInner {
	/// Variants and Collections are taken on initialized and do not change over the runtime of program
	pub variants: Vec<AssetVariant>,
	pub collections: Vec<AssetCollection>,

	/// Assets are currently kept in a vector hidden behind a RwLock
	///
	/// # Todo
	///
	/// * Faster asset lookup
	pub assets: RwLock<HashMap<UUID, AssetEntry>>,
}

/// Manager that handles assets across multiple threads.
#[derive(Clone)]
pub struct AssetManager(pub(crate) Arc<AssetManagerInner>);

impl AssetManager {
	pub fn assets(&self) -> RwLockReadGuard<HashMap<UUID, AssetEntry>> {
		self.0.assets.read().unwrap()
	}

	pub fn collections(&self) -> &Vec<AssetCollection> {
		&self.0.collections
	}

	/// TODO
	pub fn find<T: Asset>(&self, id: impl Into<UUID>) -> Option<AssetRef<T>> {
		let id = id.into();

		let read_lock = self.0.assets.read().unwrap();
		let entry = read_lock.get(&id)?;

		// Assert if the type is incorrect
		let variant = &self.0.variants[entry.variant];
		assert!(TypeId::of::<T>() == variant.type_id);

		let engine = Engine::as_ref();

		let cache_manager = engine.module::<CacheManager>().unwrap();
		let asset_cache = cache_manager.cache::<AssetCache>().unwrap();

		let path = asset_cache.uuid_to_path.get(&id)?;

		let result = AssetRef {
			arc: entry.asset.clone(), // Increment ref count
			phantom: PhantomData,
			variant: entry.variant,
			manager: self.clone(),
			path: path.clone(),
		};

		// If we're the first reference the load the asset
		if result.strong_count() == 1 {
			let mut lock = entry.asset.write().unwrap();
			let (result, dur) = {
				let file = fs::read(path).ok()?;

				let now = Instant::now();
				let result = (variant.deserialize)(&file, &path);
				let dur = Instant::now().duration_since(now).as_secs_f64() * 1000.0;
				(result, dur)
			};

			*lock = Some(result.1);
			info!(
				"[AssetManager] Loaded asset ({}) in {:.2}ms",
				path.display(),
				dur
			);
		}

		Some(result)
	}
}

impl Module for AssetManager {
	fn new() -> Self {
		let engine = Engine::as_ref();

		let cache_manager = engine.module::<CacheManager>().unwrap();
		let asset_cache = cache_manager.cache::<AssetCache>().unwrap();

		let mut assets = HashMap::with_capacity(asset_cache.uuid_to_path.len());

		let variants = engine.register::<AssetVariant>().unwrap_or_default();
		for (id, path) in asset_cache.uuid_to_path.iter() {
			let ext = path.extension().unwrap_or_default();

			let (index, _) = variants
				.iter()
				.enumerate()
				.find(|(_, v)| v.extensions.contains(&ext.to_str().unwrap()))
				.unwrap();
			let write_time = fs::metadata(path).unwrap().modified().unwrap();

			assets.insert(
				*id,
				AssetEntry {
					variant: index,
					write_time,

					asset: Arc::new(RwLock::new(None)),
				},
			);
		}

		let collections = engine.register::<AssetCollection>().unwrap_or_default();

		Self(Arc::new(AssetManagerInner {
			variants: variants,
			collections: collections,
			assets: RwLock::new(assets),
		}))
	}

	fn depends_on(builder: Builder) -> Builder {
		let base = Path::new(file!())
			.parent()
			.unwrap()
			.parent()
			.unwrap()
			.parent()
			.unwrap()
			.parent()
			.unwrap();
		let mut engine_assets = PathBuf::from(base);
		engine_assets.push("assets/");

		builder
			.module::<Logger>()
			.module::<CacheManager>()
			.register(CacheRegister::new::<AssetCache>("assets"))
			.register(AssetCollection::new(engine_assets))
	}
}
