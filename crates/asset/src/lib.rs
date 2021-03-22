#![feature(trait_alias)]
#![feature(box_syntax)]
//! This crate provides a completely thread safe asset manager which 
//! handles defining assets, loading assets, ref counting assets, and 
//! serialization.

use core::containers::{ Vec, Box, HashSet };
use log::*;

use std::any::{ TypeId, Any };
use std::path::{ Path, PathBuf };
use std::sync::{ RwLock, Arc, RwLockReadGuard, RwLockWriteGuard  };
use std::time::SystemTime;
use std::fs::{ create_dir, read_dir };
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::{ Deref, DerefMut };
use std::fmt;

static ASSET_CAT: log::Category = "Asset";

/// Trait alis for what an `Asset` can be
pub trait Asset = Any;

pub use serde::{ Serialize, Deserialize };
pub use ron::de::from_str;
pub use ron::ser::to_string;

/// Enum for asset load errors
#[derive(Debug)]
pub enum LoadError {
    FileNotFound
}

trait LoadAsset = Fn(&Path) -> Option<Box<dyn Asset>> + Send + Sync + 'static;
trait UnloadAsset = Fn(Box<dyn Asset>) + Send + Sync + 'static;

struct VariantRegister {
    type_id:    TypeId,
    extensions: HashSet<String>,
    
    load:     Box<dyn LoadAsset>,
    unload:   Box<dyn UnloadAsset>
}

impl fmt::Debug for VariantRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariantRegister")
            .field("type", &self.type_id)
            .field("extensions", &self.extensions)
            .finish()
    }
}

/// List of information about an `Asset` variant. This includes `TypeId`, 
/// load/unload functions, and extensions. Used on `AssetManager::init()`
#[derive(Debug)]
pub struct VariantRegistry {
    entries: Vec<VariantRegister>,
}

impl VariantRegistry {
    /// Returns a new VariantRegistry for building a list of asset variants
    pub const fn new() -> Self {
        Self { entries: Vec::new(), }
    }

    /// Adds a type variant to a list to be registered on `AssetManager::init()`
    /// 
    /// # Arguments
    /// 
    /// * `path` - A `PathBuf` to be added to collection entries
    /// 
    /// # Examples
    /// 
    /// ```
    /// use asset::VariantRegistry;
    /// 
    /// let mut exts = HashSet::new();
    /// exts.insert("test".to_string());
    /// 
    /// let mut variants = VariantRegistry::new();
    /// variants
    ///     .add(exts, |path| println!("Loading {:?}", path), |asset| println!("unLoading asset"));
    /// ```
    pub fn add<'a, T, Load, Unload>(&'a mut self, extensions: HashSet<String>, load: Load, unload: Unload) -> &'a mut Self where 
        T:      Asset + Sized, 
        Load:   Fn(&Path) -> Result<T, LoadError> + Send + Sync + 'static, 
        Unload: Fn(T) + Send + Sync + 'static
    {
        self.entries.push(VariantRegister{
            type_id:    TypeId::of::<T>(),
            extensions: extensions,
            
            load: box move |path| {
                let result = load(path);
                if result.is_err() {
                    return None;
                }

                let foo = box result.unwrap();

                Some(foo as Box<dyn Asset>)
            },

            unload: box move |asset| {
                let actual = asset.downcast::<T>().unwrap();
                unload(*actual); // Currently unload does not have an error out. This could change!!!!!
            },
        });
        self
    }
}

/// List of all collections. A `Collection` is defined by a path to a directory. The 
/// asset manager uses this directory and all sub directories for finding assets.
/// `AssetManager::init()`
#[derive(Debug)]
pub struct CollectionRegistry {
    entries: Vec<PathBuf>
}

impl CollectionRegistry {
    /// Returns a new CollectionRegistry
    pub const fn new() -> Self {
        Self { entries: Vec::new(), }
    }

    /// Adds a path to a list for collection registration on `AssetManager::init()`
    /// 
    /// # Arguments
    /// 
    /// * `path` - A `PathBuf` to be added to collection entries
    /// 
    /// # Examples
    /// 
    /// ```
    /// use asset::CollectionRegistry;
    /// let mut collections = CollectionRegistry::new();
    /// collections
    ///     .add(PathBuf::from("assets/"))
    ///     .add(PathBuf::from("plugin/assets"));
    /// ```
    pub fn add(&mut self, path: PathBuf) -> &mut Self {
        self.entries.push(path);
        self
    }
}

#[derive(Debug)]
struct AssetEntry {
    path:       PathBuf,
    variant:    usize, // Index into AssetManager::variants
    write_time: SystemTime,

    // Arc which is used for asset ref counting. As long as there is more 
    // than 1 reference the asset will be loaded
    asset: Arc<RwLock<Option<Box<dyn Asset>>>>,
}

/// A thread-safe reference-counting `Asset` reference.
/// 
/// Essentially an `AssetRef<T: Asset>` is an `Arc<RwLock<T>>` to an asset. As long
/// as an `Asset` has an `AssetRef` the `Asset` will be loaded. It can then be accessed 
/// like a `RwLock` with its own custom read and write guards
///
/// # Todo
///
/// * `AssetWeak<T: Asset>` for asset weak ptrs
#[derive(Clone)]
pub struct AssetRef<T: Asset + Sized> {
    arc:     Arc<RwLock<Option<Box<dyn Asset>>>>,
    phantom: PhantomData<T>,
    variant: usize, // Index into AssetManager::variants
}

impl<T: Asset + Sized> AssetRef<T> {
    /// Returns a `Option<AssetRef>
    pub fn new<P: AsRef<Path>>(p: P) -> Option<Self> {
        let asset_manager = AssetManager::as_ref();

        let read_lock = asset_manager.assets.read().unwrap();
        let entry = read_lock.iter().find(|it| it.path == p.as_ref())?;

        // Assert if the type is incorrect
        let variant = &asset_manager.variants[entry.variant];
        assert!(TypeId::of::<T>() == variant.type_id);
        
        let result = Self { 
            arc:     entry.asset.clone(), // Increment ref count
            phantom: PhantomData,
            variant: entry.variant,
        };

        // If we're the first reference the load the asset
        if result.strong_count() == 1 {
            let mut lock = entry.asset.write().unwrap();
            *lock = (variant.load)(&entry.path);
        }

        Some(result)
    }

    /// Returns an `AssetWriteGuard<T>` for RAII exclusive write access
    ///
    /// # Examples
    ///
    /// ```
    /// struct Test {
    ///     foo: i32,
    /// }
    ///
    /// let mut asset_ref: AssetRef<T> = AssetRef::new("assets/test.test").unwrap();
    /// let mut write_lock = asset_ref.write();
    /// write_lock.foo = 45;
    /// ```
    pub fn write(&mut self) -> AssetWriteGuard<T> {
        AssetWriteGuard {
            lock:    self.arc.write().unwrap(),
            phantom: PhantomData,
        }
    }

    /// Returns an `AssetReadGuard<T>` for RAII shared read access
    ///
    /// # Examples
    ///
    /// ```
    /// struct Test {
    ///     foo: i32,
    /// }
    ///
    /// let asset_ref: AssetRef<T> = AssetRef::new("assets/test.test").unwrap();
    /// let read_lock = asset_ref.read();
    /// read_lock.foo = 45;
    /// ```
    pub fn read(&self) -> AssetReadGuard<T> {
        AssetReadGuard {
            lock:    self.arc.read().unwrap(),
            phantom: PhantomData,
        }
    }

    /// Returns the number of references to `Asset`
    pub fn strong_count(&self) -> usize {
        // We always have 1 reference to the asset and what we care about here is the 
        // number of references to the asset
        Arc::strong_count(&self.arc) - 1 
    }

    /// Returns the number of weak references to `Asset`
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.arc)
    }
}

impl<T: Asset + Sized + fmt::Debug> fmt::Debug for AssetRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read_lock = self.read();

        f.debug_struct("AssetRef")
            .field("asset", &*read_lock)
            .field("strong_count", &self.strong_count())
            .field("weak_count", &self.weak_count())
            .finish()
    }
}

impl<T: Asset + Sized> Drop for AssetRef<T> {
    fn drop(&mut self) {
        // If we're the last unload the asset
        if self.strong_count() == 1 {
            let asset_manager = AssetManager::as_ref();
            let variant = &asset_manager.variants[self.variant];

            let mut lock = self.arc.write().unwrap();
            (variant.unload)(lock.take().unwrap());
        }
    }
}

/// RAII structure used to release the exclusive write access of an `AssetRef` when dropped.
/// This structure is created by the `write` method on `AssetRef`
pub struct AssetWriteGuard<'a, T: Asset + Sized> {
    lock:    RwLockWriteGuard<'a, Option<Box<dyn Asset>>>,
    phantom: PhantomData<T>
}

impl<'a, T: Asset + Sized> Deref for AssetWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.lock.as_ref().unwrap().downcast_ref::<T>().unwrap()
    }
}

impl<'a, T: Asset + Sized> DerefMut for AssetWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.lock.as_mut().unwrap().downcast_mut::<T>().unwrap()
    }
}

/// RAII structure used to release the shared read access of an `AssetRef` when dropped.
/// This structure is created by the `read` method on `AssetRef`
pub struct AssetReadGuard<'a, T: Asset + Sized> {
    lock:    RwLockReadGuard<'a, Option<Box<dyn Asset>>>,
    phantom: PhantomData<T>
}

impl<'a, T: Asset + Sized> Deref for AssetReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.lock.as_ref().unwrap().downcast_ref::<T>().unwrap()
    }
}

/// Manager that handles assets across multiple threads. 
#[derive(Debug)]
pub struct AssetManager {
    /// Variants and Collections are taken on initialized and do not change over the runtime of program
    variants:    Vec<VariantRegister>,
    collections: Vec<PathBuf>,
    
    /// Assets are currently kept in a vector hidden behind a RwLock
    /// 
    /// # Todo
    ///
    /// * Faster asset lookup
    assets: RwLock<Vec<AssetEntry>>,
}

static mut ASSET_MANAGER: Option<AssetManager> = None;

impl AssetManager {
    /// Initializes the global asset manager and discovers all current assets
    ///
    /// # Arguments
    ///
    /// * `variants` - A `VariantRegistery` which contains all variants for the runtime of the program
    /// * `collections` - A `CollectionRegistry` which containts a path to all collections
    ///
    /// # Examples
    ///
    /// ```
    /// use asset::{ AssetManager, VariantRegistry, CollectionRegistry };
    /// let variants = VariantRegistry::new();
    /// let collections = CollectionRegistry::new();
    ///
    /// AssetManager::init(variants, collections);
    /// ```
    pub fn init(variants: VariantRegistry, collections: CollectionRegistry) {
        // Start off by making sure every collection exist
        for it in collections.entries.iter() {
            if !it.exists() {
                create_dir(it).unwrap();
                log!(ASSET_CAT, "Created collection directory {:?}", it);
            }
        }

        // Initialize the asset manager and grab a mutable ref
        // UNSAFE: Grabbing a mut ref to the global state. Will be safe due to 
        //      no assets being used before initialization
        let asset_manager: &mut AssetManager;
        unsafe {
            ASSET_MANAGER = Some(AssetManager{
                variants:    variants.entries,
                collections: collections.entries,
                assets:      RwLock::new(Vec::new()),
            });
            asset_manager = ASSET_MANAGER.as_mut().unwrap()
        }

        // Recusrive file directory lookup. Sending members instead of AssetManager is due to RWLock on assets
        fn discover(mut path: PathBuf, assets: &mut Vec<AssetEntry>, variants: &Vec<VariantRegister>) -> PathBuf {
            // Iterate through every entry in a directory
            for entry in read_dir(path.as_path()).unwrap() {
                let entry = entry.unwrap();
                let file_type = entry.file_type().unwrap();
                
                // If the entry is a directory then recurse through it
                if file_type.is_dir() {
                    path.push(entry.file_name());
                    path = discover(path, assets, variants);
                    path.pop();
                } else {
                    let path = entry.path();
                    let ext = path.extension().unwrap_or(OsStr::new(""));
                    
                    // Find variant from extension
                    let v = variants.iter().enumerate()
                        .find(|v| v.1.extensions.get(ext.to_str().unwrap()).is_some());
                    if v.is_none() { continue; }
                    let (v_index, _) = v.unwrap();
                    
                    // Build the asset entry and push to assets vector
                    let write_time = entry.metadata().unwrap().modified().unwrap();
                    assets.push(AssetEntry{
                        path:       path,
                        variant:    v_index,
                        
                        asset:      Arc::new(RwLock::new(None)),
                        write_time: write_time,
                    });
                }
            }
            
            path
        }
        
        // Run through each collection recursively discovering assets
        let mut assets_lock = asset_manager.assets.write().unwrap();
        for it in asset_manager.collections.iter() { 
            log!(ASSET_CAT, "Discovering assets in {:?}", it);
            discover(it.clone(), &mut *assets_lock, &asset_manager.variants); 
        }
    }

    /// Returns the global `AssetManager` as an immutable reference
    pub fn as_ref() -> &'static AssetManager {
        unsafe{ ASSET_MANAGER.as_ref().unwrap() }
    }
}