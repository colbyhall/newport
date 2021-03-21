#![feature(trait_alias)]
#![feature(box_syntax)]
//! This crate provides an asset manager which handles defining
//! assets, loading assets, ref counting assets, and serialization 

use core::containers::{ Vec, Box, HashSet };
use log::*;

use std::any::{ TypeId, Any };
use std::path::{ Path, PathBuf };
use std::sync::{ Mutex, RwLock, Arc, RwLockReadGuard, RwLockWriteGuard  };
use std::time::SystemTime;
use std::fs::{ create_dir, read_dir };
use std::ffi::OsStr;
use std::marker::PhantomData;
use std::ops::{ Deref, DerefMut };
use std::fmt;

static ASSET_CAT: log::Category = "Asset";

pub trait Asset = Any;

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

/// List of information about an `Asset` variant. This includes `TypeId`, load/unload functions, and extensions
#[derive(Debug)]
pub struct VariantRegistry {
    entries: Vec<VariantRegister>,
}

impl VariantRegistry {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add<'a, T, Load, Unload>(&'a mut self, extensions: HashSet<String>, load: Load, unload: Unload) -> &'a mut Self where 
        T:      Asset + Sized, 
        Load:   Fn(&Path) -> Result<T, LoadError> + Send + Sync + 'static, 
        Unload: Fn(T) + Send + Sync + 'static
    {
        self.entries.push(VariantRegister{
            type_id:    TypeId::of::<T>(),
            extensions: extensions,
            
            load:   box move |path| {
                let result = load(path);
                if result.is_err() {
                    return None;
                }

                let foo = box result.unwrap();

                Some(foo as Box<dyn Asset>)
            },
            unload: box move |asset| {
                let actual = asset.downcast::<T>().unwrap();
                unload(*actual); // Current unload does not have an error out. This could change!!!!!
            },
        });
        self
    }
}

/// List of all collections. A `Collection` is defined by a path to a directory. The asset manager uses this 
/// directory and all sub directories for finding assets.
#[derive(Debug)]
pub struct CollectionRegistry {
    entries: Vec<PathBuf>
}

impl CollectionRegistry {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(mut self, path: PathBuf) -> Self {
        self.entries.push(path);
        self
    }
}

#[derive(Debug)]
struct AssetEntry {
    // Static data that never changes after registration
    path: PathBuf,
    variant: usize, // Index into AssetManager::variants

    // Custom RWLock synchronized by atomics and AssetRef
    asset:      Arc<RwLock<Option<Box<dyn Asset>>>>,
    write_time: Mutex<SystemTime>,
}

pub struct AssetRef<T: Asset + Sized> {
    arc:     Arc<RwLock<Option<Box<dyn Asset>>>>,
    phantom: PhantomData<T>,
}

impl<T: Asset + Sized + fmt::Debug> AssetRef<T> {
    pub fn new<P: AsRef<Path>>(p: P) -> Option<Self> {
        let asset_manager = AssetManager::as_ref();

        let read_lock = asset_manager.assets.read().unwrap();
        let entry = read_lock.iter().find(|it|it.path == p.as_ref())?;

        // Assert if the type is incorrect
        let variant = &asset_manager.variants[entry.variant];
        assert!(TypeId::of::<T>() == variant.type_id);
        
        let result = Self { 
            arc:     entry.asset.clone(), // Increment ref count
            phantom: PhantomData
        };

        // The entry always has a reference to the asset arc. So if
        // we're the 2nd strong reference then we need to actually load this asset
        if result.strong_count() == 2 {
            let mut lock = entry.asset.write().unwrap();
            *lock = (variant.load)(&entry.path);

            println!("{:?}", lock.deref());
        }

        Some(result)
    }

    pub fn write(&self) -> AssetWriteGuard<T> {
        AssetWriteGuard {
            lock:    self.arc.write().unwrap(),
            phantom: PhantomData,
        }
    }

    pub fn read(&self) -> AssetReadGuard<T> {
        AssetReadGuard {
            lock:    self.arc.read().unwrap(),
            phantom: PhantomData,
        }
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.arc)
    }

    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.arc)
    }
}

impl<T: Asset + Sized + fmt::Debug> fmt::Debug for  AssetRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let read_lock = self.read();

        f.debug_struct("AssetRef")
            .field("asset", &*read_lock)
            .field("strong_count", &self.strong_count())
            .field("weak_count", &self.weak_count())
            .finish()
    }
}

pub struct AssetWriteGuard<'a, T: Asset + Sized> {
    lock:    RwLockWriteGuard<'a, Option<Box<dyn Asset>>>,
    phantom: PhantomData<T>
}

impl<'a, T: Asset + Sized> Deref for AssetWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        let asset_ptr = self.lock.deref().as_ref().unwrap() as *const dyn Asset;

        // UNSAFE: Write guard ensures that asset is loaded and is safe to access
        unsafe{ &*(asset_ptr as *const T) }
    }
}

impl<'a, T: Asset + Sized> DerefMut for AssetWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        let asset_ptr = self.lock.deref_mut().as_mut().unwrap() as *mut dyn Asset;

        // UNSAFE: Write guard ensures that asset is loaded and is safe to access
        unsafe{ &mut *(asset_ptr as *mut T) }
    }
}

pub struct AssetReadGuard<'a, T: Asset + Sized> {
    lock:    RwLockReadGuard<'a, Option<Box<dyn Asset>>>,
    phantom: PhantomData<T>
}

impl<'a, T: Asset + Sized> Deref for AssetReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // UNSAFE: Read guard ensures that asset is loaded and is safe to access
        let asset_ptr = self.lock.deref().as_ref().unwrap() as *const dyn Asset;
        println!("{:?}", asset_ptr);
        unsafe{ &*(asset_ptr as *const T) }
    }
}

/// Manager that handles assets across multiple threads. 
#[derive(Debug)]
pub struct AssetManager {
    variants:    Vec<VariantRegister>,
    collections: Vec<PathBuf>,
    
    assets: RwLock<Vec<AssetEntry>>,
}

static mut ASSET_MANAGER: Option<AssetManager> = None;

impl AssetManager {
    /// Initializes the global asset manager and discovers all current assets
    ///
    /// # Arguments
    ///
    /// * 'variants' - A `VariantRegistery` which contains all variants for the runtime of the program
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
        let asset_manager: &mut AssetManager;
        unsafe {
            ASSET_MANAGER = Some(AssetManager{
                variants:    variants.entries,
                collections: collections.entries,
                assets:      RwLock::new(Vec::new()),
            });
            asset_manager = ASSET_MANAGER.as_mut().unwrap()
        }

        // Recusrive file directory lookup
        fn discover(mut path: PathBuf, assets: &mut Vec<AssetEntry>) -> PathBuf {
            for entry in read_dir(path.as_path()).unwrap() {
                let entry = entry.unwrap();
                
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    path.push(entry.file_name());
                    path = discover(path, assets);
                    path.pop();
                } else {
                    let path = entry.path();
                    let ext = path.extension().unwrap_or(OsStr::new(""));
                    
                    let variants = unsafe{ &ASSET_MANAGER.as_mut().unwrap().variants };
                    
                    // Find variant from extension
                    let v = variants.iter().enumerate().find(|v| v.1.extensions.get(ext.to_str().unwrap()).is_some());
                    if v.is_none() { continue; }
                    let (v_index, _) = v.unwrap();
                    
                    // Build the asset entry and push to assets vector
                    let write_time = entry.metadata().unwrap().modified().unwrap();
                    assets.push(AssetEntry{
                        path:       path,
                        variant:    v_index,
                        
                        asset:      Arc::new(RwLock::new(None)),
                        write_time: Mutex::new(write_time),
                    });
                }
            }
            
            path
        }
        
        // Run through each collection recursively discovering assets
        let mut assets_lock = asset_manager.assets.write().unwrap();
        for it in asset_manager.collections.iter() { 
            log!(ASSET_CAT, "Discovering assets in {:?}", it);
            discover(it.clone(), &mut *assets_lock); 
        }
    }

    pub fn as_ref() -> &'static AssetManager {
        unsafe{ ASSET_MANAGER.as_ref().unwrap() }
    }
}