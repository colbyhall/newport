//! This crate defines the general data system for the entire engine.
//! This system provides a thread safe resource manager that handles
//! ref counting, serialization, and garbage collection of resources.

use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
	error::Error,
	fmt,
	fs,
	fs::File,
	marker::PhantomData,
	path::{
		Path,
		PathBuf,
	},
	sync::{
		Arc,
		Mutex,
		RwLock,
		Weak,
	},
	time::Instant,
};

use serde::{
	self,
	bincode,
	de::DeserializeOwned,
	ron,
	Deserialize,
	Serialize,
};

use cache::{
	Cache,
	CacheManager,
	CacheRef,
};

use engine::{
	info,
	Builder,
	Engine,
	Module,
	Uuid,
};

pub use resources_derive::*;

#[derive(Clone)]
pub struct ResourceVariant {
	pub(crate) type_id: TypeId,
}

pub trait Resource: 'static {
	fn default_uuid() -> Option<Uuid> {
		None
	}

	fn variant() -> ResourceVariant {
		ResourceVariant {
			type_id: TypeId::of::<Self>(),
		}
	}
}

#[derive(Debug)]
pub enum RefError {
	NoManager,
	NotFound(Uuid),
	IncorrectType { expected: TypeId, found: TypeId },
}

impl fmt::Display for RefError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl Error for RefError {}

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Ref<T: Resource> {
	arc: Arc<Box<dyn Any>>,
	phantom: PhantomData<T>,
	uuid: Uuid,
	path: Option<PathBuf>,
}

impl<T: Resource> Ref<T> {
	pub fn find(uuid: impl Into<Uuid>) -> Result<Ref<T>> {
		let manager: &ResourceManager = Engine::module().ok_or(RefError::NoManager)?;

		let uuid = uuid.into();
		let resources = manager.resources.read().unwrap();
		let mut entry = resources
			.get(&uuid)
			.ok_or(RefError::NotFound(uuid))?
			.lock()
			.unwrap();

		if entry.variant != TypeId::of::<T>() {
			return Err(Box::new(RefError::IncorrectType {
				expected: TypeId::of::<T>(),
				found: entry.variant,
			}));
		}

		if let Some(resource) = entry.resource.upgrade() {
			Ok(Ref {
				arc: resource,
				phantom: PhantomData,
				uuid,
				path: entry.path.clone(),
			})
		} else {
			let importer = match entry.importer {
				Some(importer) => importer,
				None => return Err(Box::new(RefError::NotFound(uuid))),
			};

			let path_cache = CacheRef::<PathCache>::new().unwrap();
			let path = path_cache
				.uuid_to_path
				.get(&uuid)
				.ok_or(RefError::NotFound(uuid))?;

			let importer_variant = manager
				.importer_variants_by_type
				.get(&importer)
				.ok_or(RefError::NotFound(uuid))?;

			let now = Instant::now();
			let resource_file = fs::read(path)?;

			let mut meta_path = path.clone().into_os_string();
			meta_path.push(crate::META_EXTENSION);

			// TODO: Maybe cache the meta files
			// SPEED: Reading 2 files per resource
			let meta_file = fs::read(meta_path)?;
			let meta = (importer_variant.load_meta)(&meta_file[..])?.1;

			let arc = Arc::new((importer_variant.load_resource)(&meta, &resource_file[..])?);
			let dur = Instant::now().duration_since(now).as_secs_f64() * 1000.0;

			info!("Loaded resource ({}) in {:.2}ms", path.display(), dur);

			let result = Ref {
				arc,
				phantom: PhantomData,
				uuid,
				path: entry.path.clone(),
			};

			entry.resource = Arc::downgrade(&result.arc);

			Ok(result)
		}
	}

	/// Returns the number of references to the `Resource`
	pub fn strong_count(&self) -> usize {
		Arc::strong_count(&self.arc)
	}

	/// Returns the number of weak references to the `Resource`
	pub fn weak_count(&self) -> usize {
		Arc::weak_count(&self.arc)
	}

	/// Returns the Uuid of the 'Resource'
	pub fn uuid(&self) -> Uuid {
		self.uuid
	}

	/// Returns the Uuid of the 'Resource'
	pub fn path(&self) -> Option<&Path> {
		match &self.path {
			Some(path) => Some(path),
			None => None,
		}
	}
}

unsafe impl<T: Resource> Sync for Ref<T> {}
unsafe impl<T: Resource> Send for Ref<T> {}

pub(crate) const META_EXTENSION: &str = ".meta";

/// TODO: Document
#[derive(Clone)]
pub struct ImporterVariant {
	importer: TypeId,
	resource: TypeId,

	extensions: Vec<&'static str>,

	#[allow(clippy::type_complexity)]
	load_resource: fn(&Box<dyn Any>, &[u8]) -> Result<Box<dyn Any>>,
	#[allow(clippy::type_complexity)]
	load_meta: fn(&[u8]) -> Result<(Uuid, Box<dyn Any>)>,

	#[allow(clippy::type_complexity)]
	save_resource: fn(&Box<dyn Any>, &Box<dyn Any>, &mut File) -> Result<()>,
	#[allow(clippy::type_complexity)]
	save_meta: fn(Uuid, &Box<dyn Any>, &mut File) -> Result<()>,
}

/// TODO: Document
pub trait Importer: Sized + Serialize + DeserializeOwned + 'static {
	type Target: Resource;
	const EXTENSIONS: &'static [&'static str];

	fn import(&self, bytes: &[u8]) -> Result<Self::Target>;
	fn export(&self, resource: &Self::Target, file: &mut File) -> Result<()>;

	fn variant() -> ImporterVariant {
		fn load_resource<T: Importer>(meta: &Box<dyn Any>, bytes: &[u8]) -> Result<Box<dyn Any>> {
			let meta = meta.downcast_ref::<T>().unwrap();
			Ok(Box::new(meta.import(bytes)?))
		}

		fn load_meta<T: Importer>(bytes: &[u8]) -> Result<(Uuid, Box<dyn Any>)> {
			#[derive(Serialize, Deserialize)]
			#[serde(rename = "Meta")]
			struct MetaFile<T> {
				uuid: Uuid,
				#[serde(bound(deserialize = "T: Deserialize<'de>"))]
				importer: T,
			}

			let contents = std::str::from_utf8(bytes)?;
			let meta: MetaFile<T> = ron::from_str(contents)?;
			Ok((meta.uuid, Box::new(meta.importer)))
		}

		fn save_resource<T: Importer>(
			meta: &Box<dyn Any>,
			resource: &Box<dyn Any>,
			file: &mut File,
		) -> Result<()> {
			let meta = meta.downcast_ref::<T>().unwrap();
			let resource = resource.downcast_ref::<T::Target>().unwrap();

			meta.export(resource, file)
		}

		fn save_meta<T: Importer>(uuid: Uuid, meta: &Box<dyn Any>, file: &mut File) -> Result<()> {
			let importer = meta.downcast_ref::<T>().unwrap();

			#[derive(Serialize)]
			#[serde(rename = "Meta")]
			struct MetaFile<'a, T> {
				uuid: Uuid,
				#[serde(bound(serialize = "T: Serialize"))]
				importer: &'a T,
			}

			let meta = MetaFile { uuid, importer };
			ron::ser::to_writer(file, &meta)?;
			Ok(())
		}

		ImporterVariant {
			importer: TypeId::of::<Self>(),
			resource: TypeId::of::<Self::Target>(),

			extensions: Self::EXTENSIONS.to_owned(),

			load_resource: load_resource::<Self>,
			load_meta: load_meta::<Self>,

			save_resource: save_resource::<Self>,
			save_meta: save_meta::<Self>,
		}
	}
}

/// TODO: Document
#[derive(Clone)]
pub struct Collection {
	path: PathBuf,
}

impl Collection {
	pub fn new(path: impl Into<PathBuf>) -> Collection {
		Collection { path: path.into() }
	}

	pub fn path(&self) -> &Path {
		&self.path
	}
}

/// TODO: Document
#[derive(Serialize, Deserialize, Debug)]
struct PathCache {
	uuid_to_path: HashMap<Uuid, PathBuf>,
}

impl Cache for PathCache {
	fn new() -> Self {
		let collections = Engine::register::<Collection>().unwrap();

		let mut variants = HashMap::new();
		let importer_variants = Engine::register::<ImporterVariant>()
			.expect("Resource Manager is required as a dependency but no ImporterVariants have been registered.");

		// TODO: Look into a functional way of doing this
		for variant in importer_variants.iter() {
			for ext in variant.extensions.iter() {
				variants.insert(*ext, variant.clone());
			}
		}

		// Run through all the collections and create a directory if one is not created
		for it in collections.iter() {
			if !it.path.exists() {
				fs::create_dir(&it.path).unwrap();
				info!("Created collection directory ({})", it.path.display());
			}
		}

		// Recursive directory reader
		fn discover(
			mut path: PathBuf,
			uuid_to_path: &mut HashMap<Uuid, PathBuf>,
			variants: &HashMap<&'static str, ImporterVariant>,
		) -> PathBuf {
			for entry in fs::read_dir(&path).unwrap() {
				let entry = entry.unwrap();
				let file_type = entry.file_type().unwrap();

				if file_type.is_dir() {
					path.push(entry.file_name());
					path = discover(path, uuid_to_path, variants);
					path.pop();
				} else if file_type.is_file() {
					let path = entry.path();

					let variant = {
						let ext = path
							.extension()
							.unwrap_or_default()
							.to_str()
							.unwrap_or_default();
						variants.get(ext)
					};
					if let Some(variant) = variant {
						let mut meta_path = path.clone().into_os_string();
						meta_path.push(crate::META_EXTENSION);

						let contents = match fs::read(&meta_path) {
							Ok(contents) => contents,
							_ => continue,
						};
						info!("Caching resource ({})", path.display());
						let uuid = (variant.load_meta)(&contents).unwrap().0;

						uuid_to_path.insert(uuid, path);
					}
				} else {
					continue;
				}
			}

			path
		}

		let mut uuid_to_path = HashMap::new();
		for it in collections.iter() {
			info!("Discovering resources in ({})", it.path.display());
			discover(it.path.clone(), &mut uuid_to_path, &mut variants);
		}

		Self { uuid_to_path }
	}

	fn needs_reload(&self) -> bool {
		false
	}
}

// TODO: Set this up in a way that keeps everything tightly packed. Also do GC
struct ResourceEntry {
	path: Option<PathBuf>,
	importer: Option<TypeId>,

	variant: TypeId,
	resource: Weak<Box<dyn Any>>,
}

/// TODO: Document
pub struct ResourceManager {
	resource_variants: HashMap<TypeId, ResourceVariant>,
	collections: Vec<Collection>,

	importer_variants_by_extension: HashMap<&'static str, ImporterVariant>,
	importer_variants_by_type: HashMap<TypeId, ImporterVariant>,

	// TODO: Make adding and destroying resources lockless
	resources: RwLock<HashMap<Uuid, Mutex<ResourceEntry>>>,
}

impl Module for ResourceManager {
	fn new() -> Self {
		let resource_variants = Engine::register::<ResourceVariant>()
			.expect("Resource Manager is required as a dependency but no ResourceVariants have been registered.")
			.iter()
			.map(|x| (x.type_id, x.clone()))
			.collect();

		let importer_variants = Engine::register::<ImporterVariant>()
			.expect("Resource Manager is required as a dependency but no ImporterVariants have been registered.");

		// TODO: Look into a functional way of doing this
		let mut importer_variants_by_extension = HashMap::new();
		for variant in importer_variants.iter() {
			for ext in variant.extensions.iter() {
				importer_variants_by_extension.insert(*ext, variant.clone());
			}
		}

		let path_cache = CacheRef::<PathCache>::new().unwrap();
		let mut resources = HashMap::with_capacity(path_cache.uuid_to_path.len());

		for (id, path) in path_cache.uuid_to_path.iter() {
			let ext = path
				.extension()
				.unwrap_or_default()
				.to_str()
				.unwrap_or_default();

			// Skip any path found without a proper extension. This is so we
			// don't crash when an asset register is removed.
			let importer_variant = match importer_variants_by_extension.get(ext) {
				Some(v) => v,
				None => continue,
			};

			resources.insert(
				*id,
				Mutex::new(ResourceEntry {
					path: Some(path.clone()),
					importer: Some(importer_variant.importer),

					variant: importer_variant.resource,
					resource: Weak::new(),
				}),
			);
		}

		Self {
			resource_variants,
			collections: Engine::register().unwrap().clone(),

			importer_variants_by_extension,
			importer_variants_by_type: importer_variants
				.iter()
				.map(|x| (x.importer, x.clone()))
				.collect(),
			resources: RwLock::new(resources),
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
		engine_assets.push("contents/");
		builder
			.module::<CacheManager>()
			.register(Collection::new(engine_assets))
			.register(PathCache::variant())
	}
}
