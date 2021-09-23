use crate::Asset;
use crate::Uuid;
use crate::{
	AssetManager,
	PathCache,
	Result,
	ASSET_MANAGER_CATEGORY,
};

use engine::{
	info,
	Engine,
};

use cache::CacheRef;

use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;
use std::{
	any::{
		Any,
		TypeId,
	},
	fmt,
	fs,
	marker::PhantomData,
	ops::Deref,
	sync::Arc,
};

use serde::{
	self,
	Deserialize,
	Deserializer,
	Serialize,
	Serializer,
};

#[derive(Debug)]
pub enum AssetRefError {
	NoManager,
	NotFound,
}

impl fmt::Display for AssetRefError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Error for AssetRefError {}

pub struct AssetRef<T: Asset> {
	pub(crate) arc: Arc<Box<dyn Any>>,
	pub(crate) phantom: PhantomData<T>,
	pub(crate) uuid: Uuid,
}

unsafe impl<T: Asset> Sync for AssetRef<T> {}
unsafe impl<T: Asset> Send for AssetRef<T> {}

impl<T: Asset> AssetRef<T> {
	pub fn new(id: impl Into<Uuid>) -> Result<AssetRef<T>> {
		let manager: &AssetManager = Engine::module().ok_or(AssetRefError::NoManager)?;

		let id = id.into();
		let assets = manager.assets.read().unwrap();
		let entry = assets.get(&id).ok_or(AssetRefError::NotFound)?;

		let mut asset = entry.asset.lock().unwrap();
		if let Some(asset) = asset.upgrade() {
			Ok(AssetRef {
				arc: asset,
				phantom: PhantomData,
				uuid: id,
			})
		} else {
			let asset_cache = CacheRef::<PathCache>::new().unwrap();
			let path = asset_cache
				.uuid_to_path
				.get(&id)
				.ok_or(AssetRefError::NotFound)?;

			// Assert if the type is incorrect
			let variant = &manager.variants[entry.variant];
			assert!(TypeId::of::<T>() == variant.asset);

			let now = Instant::now();
			let asset_file = fs::read(path)?;

			let mut meta_path = path.clone().into_os_string();
			meta_path.push(crate::META_EXTENSION);

			let meta_file = fs::read(meta_path)?;
			let meta = (variant.load_meta)(&meta_file[..])?.1;

			let arc = Arc::new((variant.load_asset)(&meta, &asset_file[..])?);
			let dur = Instant::now().duration_since(now).as_secs_f64() * 1000.0;

			info!(
				ASSET_MANAGER_CATEGORY,
				"Loaded asset ({}) in {:.2}ms",
				path.display(),
				dur
			);

			let result = AssetRef {
				arc,
				phantom: PhantomData,
				uuid: id,
			};

			*asset = Arc::downgrade(&result.arc);

			Ok(result)
		}
	}

	/// Returns the number of references to `Asset`
	pub fn strong_count(&self) -> usize {
		Arc::strong_count(&self.arc)
	}

	/// Returns the number of weak references to `Asset`
	pub fn weak_count(&self) -> usize {
		Arc::weak_count(&self.arc)
	}

	pub fn path(&self) -> PathBuf {
		let asset_cache = CacheRef::<PathCache>::new().unwrap();
		asset_cache.uuid_to_path.get(&self.uuid).unwrap().clone()
	}
}

impl<T: Asset> Deref for AssetRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.arc.downcast_ref().unwrap()
	}
}

impl<T: Asset + fmt::Debug> fmt::Debug for AssetRef<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("AssetRef")
			.field("asset", &*self)
			.field("strong_count", &self.strong_count())
			.field("weak_count", &self.weak_count())
			.finish()
	}
}

impl<T: Asset> Clone for AssetRef<T> {
	fn clone(&self) -> Self {
		Self {
			arc: self.arc.clone(),
			uuid: self.uuid,
			phantom: PhantomData,
		}
	}
}

impl<T: Asset> PartialEq for AssetRef<T> {
	fn eq(&self, rhs: &Self) -> bool {
		self.uuid == rhs.uuid
	}
}

impl<T: Asset> Serialize for AssetRef<T> {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		Serialize::serialize(&self.uuid, serializer)
	}
}

impl<'de, T: Asset> Deserialize<'de> for AssetRef<T> {
	#[allow(clippy::many_single_char_names)]
	fn deserialize<D>(deserializer: D) -> std::result::Result<AssetRef<T>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let uuid: Uuid = Deserialize::deserialize(deserializer)?;
		Ok(AssetRef::new(uuid).unwrap_or_default())
	}
}

impl<T: Asset> Default for AssetRef<T> {
	fn default() -> AssetRef<T> {
		let uuid = T::default_uuid().unwrap_or_else(|| {
			panic!(
				"Asset of type {} has no default_uuid",
				std::any::type_name::<T>()
			)
		});

		AssetRef::new(uuid).unwrap_or_else(|err| {
			panic!(
				"Asset of type {} has default_uuid but can not load asset due to {:?}",
				std::any::type_name::<T>(),
				err
			)
		})
	}
}
