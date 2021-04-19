#![feature(box_syntax)]
#![feature(trait_alias)]
//! This crate provides a completely thread safe asset manager which 
//! handles defining assets, loading assets, ref counting assets, and 
//! serialization.

use newport_core::containers::{ Vec, Box };
use newport_engine::*;
use newport_log::*;

use std::any::{ TypeId, Any };
use std::sync::{ RwLock, Arc, RwLockReadGuard, RwLockWriteGuard, Mutex };
use std::time::SystemTime;
use std::fs::{ create_dir, read_dir };
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::{ Deref, DerefMut };
use std::fmt;
use std::convert::Into;

/// Trait alis for what an `Asset` can be
pub trait Asset: Sized + 'static {
    fn load(path: &Path) -> Result<Self, LoadError>;
    fn unload(_asset: Self) { }
    fn extension() -> &'static str;
}

pub use std::path::{ Path, PathBuf };

pub use ron::de;
pub use ron::ser;

/// Enum for asset load errors
#[derive(Debug)]
pub enum LoadError {
    FileNotFound,
    ParseError,
    DataError
}

struct VariantRegister {
    type_id:    TypeId,
    extension:  &'static str,
    
    load:     fn(&Path) -> Result<Box<dyn Any>, LoadError>,
    unload:   fn(Box<dyn Any>)
}

#[derive(Debug)]
struct AssetEntry {
    path:       PathBuf,
    variant:    usize, // Index into AssetManager::variants
    write_time: SystemTime,

    // Arc which is used for asset ref counting. As long as there is more 
    // than 1 reference the asset is ensured to be loaded
    asset: Arc<RwLock<Option<Box<dyn Any>>>>,
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
pub struct AssetRef<T: 'static> {
    arc:     Arc<RwLock<Option<Box<dyn Any>>>>,
    phantom: PhantomData<T>,
    variant: usize, // Index into AssetManager::variants
    manager: AssetManager, 
}

impl<T: 'static> AssetRef<T> {
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
    pub fn write(&self) -> AssetWriteGuard<T> {
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

impl<T:'static + fmt::Debug> fmt::Debug for AssetRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read_lock = self.read();

        f.debug_struct("AssetRef")
            .field("asset", &*read_lock)
            .field("strong_count", &self.strong_count())
            .field("weak_count", &self.weak_count())
            .finish()
    }
}

impl<T:'static> Drop for AssetRef<T> {
    fn drop(&mut self) {
        // If we're the last unload the asset
        if self.strong_count() == 1 {
            let variants = self.manager.0.variants.lock().unwrap();
            let variant = &variants[self.variant];

            let mut lock = self.arc.write().unwrap();
            (variant.unload)(lock.take().unwrap());
        }
    }
}

impl<T:'static> Clone for AssetRef<T> {
    fn clone(&self) -> Self {
        Self {
            arc:        self.arc.clone(),
            phantom:    PhantomData,
            variant:    self.variant,
            manager:    self.manager.clone(),
        }
    }
}

/// RAII structure used to release the exclusive write access of an `AssetRef` when dropped.
/// This structure is created by the `write` method on `AssetRef`
pub struct AssetWriteGuard<'a, T: 'static> {
    lock:    RwLockWriteGuard<'a, Option<Box<dyn Any>>>,
    phantom: PhantomData<T>
}

impl<'a, T: 'static> Deref for AssetWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.lock.as_ref().unwrap().downcast_ref::<T>().unwrap()
    }
}

impl<'a, T: 'static> DerefMut for AssetWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.lock.as_mut().unwrap().downcast_mut::<T>().unwrap()
    }
}

/// RAII structure used to release the shared read access of an `AssetRef` when dropped.
/// This structure is created by the `read` method on `AssetRef`
pub struct AssetReadGuard<'a, T: 'static> {
    lock:    RwLockReadGuard<'a, Option<Box<dyn Any>>>,
    phantom: PhantomData<T>
}

impl<'a, T: 'static> Deref for AssetReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.lock.as_ref().unwrap().downcast_ref::<T>().unwrap()
    }
}

struct AssetManagerInner {
    /// Variants and Collections are taken on initialized and do not change over the runtime of program
    variants:    Mutex<Vec<VariantRegister>>,
    collections: Mutex<Vec<PathBuf>>,
    
    /// Assets are currently kept in a vector hidden behind a RwLock
    /// 
    /// # Todo
    ///
    /// * Faster asset lookup
    assets: RwLock<Vec<AssetEntry>>,
}

/// Manager that handles assets across multiple threads. 
#[derive(Clone)]
pub struct AssetManager(Arc<AssetManagerInner>);

impl AssetManager {
    /// Discovers all assets using the registered collection and variants
    pub fn discover(&self) {
        let collections = self.0.collections.lock().unwrap();
        // Start off by making sure every collection exist
        for it in collections.iter() {
            if !it.exists() {
                create_dir(it).unwrap();
                info!("[AssetManager] Created collection directory {:?}", it);
            }
        }

        // Recursive file directory lookup. Sending members instead of AssetManager is due to RWLock on assets
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
                        .find(|v| v.1.extension == ext.to_str().unwrap());
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
        
        let variants = self.0.variants.lock().unwrap();

        // Run through each collection recursively discovering assets
        let mut assets_lock = self.0.assets.write().unwrap();
        for it in collections.iter() { 
            info!("[AssetManager] Discovering assets in {:?}", it);
            discover(it.clone(), &mut *assets_lock, &variants); 
        }
    }

    /// TODO
    pub fn find<P: AsRef<Path> + fmt::Debug, T: Asset>(&self, p: P) -> Option<AssetRef<T>> {
        let read_lock = self.0.assets.read().unwrap();
        let entry = read_lock.iter().find(|it| it.path == p.as_ref())?;

        // Assert if the type is incorrect
        let variants = self.0.variants.lock().unwrap();
        let variant = &variants[entry.variant];
        assert!(TypeId::of::<T>() == variant.type_id);
        
        let result = AssetRef { 
            arc:     entry.asset.clone(), // Increment ref count
            phantom: PhantomData,
            variant: entry.variant,
            manager: self.clone(),
        };

        // If we're the first reference the load the asset
        if result.strong_count() == 1 {
            let mut lock = entry.asset.write().unwrap();
            let result = (variant.load)(&entry.path);
            if result.is_err() {
                let err = result.err().unwrap();
                error!("[AssetManager] Failed to load {:?} due to {:?}", p, err);
            } else {
                *lock = Some(result.unwrap());
            }
        }

        Some(result)
    }

    /// Adds a type variant to be used when discovering assets in [`AssetManager::discover`]
    pub fn register_variant<T: Asset>(&self) -> &Self {
        fn load<T: Asset>(path: &Path) -> Result<Box<dyn Any>, LoadError> {
            let t = T::load(path)?;
            Ok(Box::new(t))
        }

        fn unload<T: Asset>(asset: Box<dyn Any>) {
            let t = asset.downcast::<T>().unwrap();
            T::unload(*t);
        }

        let mut variants = self.0.variants.lock().unwrap();
        variants.push(VariantRegister{
            type_id:    TypeId::of::<T>(),

            extension:  T::extension(),
            load:       load::<T>,
            unload:     unload::<T>,
        });
        self
    }

    /// Adds a path to recursively search for assets in when doing [`AssetManager::discover`]
    /// 
    /// # Arguments
    /// 
    /// * `path` - A `PathBuf` to be added to collection entries
    pub fn register_collection(&self, path: impl Into<PathBuf>) -> &Self {
        let mut collections = self.0.collections.lock().unwrap();
        collections.push(path.into());
        self
    }
}

impl Module for AssetManager {
    fn new() -> Self {
        let result = Self(Arc::new(AssetManagerInner{
            variants:    Mutex::new(Vec::new()),
            collections: Mutex::new(Vec::new()),
            assets:      RwLock::new(Vec::new()),
        }));

        result.register_collection("assets/");

        result
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Logger>()
            .post_init(|engine: &Engine| {
                let asset_manager = engine.module::<AssetManager>().unwrap();
                asset_manager.discover();
            })
    }
}