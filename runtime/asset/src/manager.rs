use crate::{
	Collection,
	PathCache,
	Uuid,
	Variant,
};

use cache::Cache;
use cache::CacheRef;
use engine::{
	Builder,
	Engine,
	Module,
};

use cache::CacheManager;

use std::sync::Mutex;
use std::sync::Weak;
use std::{
	any::Any,
	collections::HashMap,
	// fs,
	path::{
		Path,
		PathBuf,
	},
	sync::RwLock,
	// time::SystemTime,
};

pub(crate) struct AssetEntry {
	pub variant: usize, // Index into AssetManager::variants
	pub asset: Mutex<Weak<Box<dyn Any>>>,
}

/// Manager that handles assets across multiple threads.
pub struct AssetManager {
	pub variants: Vec<Variant>,
	pub collections: Vec<Collection>,

	pub(crate) assets: RwLock<HashMap<Uuid, AssetEntry>>,
}

impl Module for AssetManager {
	fn new() -> Self {
		let asset_cache = CacheRef::<PathCache>::new().unwrap();

		let mut assets = HashMap::with_capacity(asset_cache.uuid_to_path.len());

		let variants = Engine::register::<Variant>().unwrap();
		for (id, path) in asset_cache.uuid_to_path.iter() {
			let ext = path.extension().unwrap_or_default();

			let variant = variants
				.iter()
				.enumerate()
				.find(|(_, v)| v.extensions.contains(&ext.to_str().unwrap()));

			// Skip any path found without a proper extension. This is so we
			// don't crash when an asset register is removed.
			let index = match variant {
				Some((index, _)) => index,
				_ => continue,
			};

			// let write_time = fs::metadata(path).unwrap().modified().unwrap();

			assets.insert(
				*id,
				AssetEntry {
					variant: index,
					// write_time,
					asset: Mutex::new(Weak::new()),
				},
			);
		}

		let collections = Engine::register::<Collection>().unwrap();

		Self {
			variants: variants.clone(),
			collections: collections.clone(),
			assets: RwLock::new(assets),
		}
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
			.module::<CacheManager>()
			.register(PathCache::variant())
			.register(Collection::new(engine_assets))
	}
}
