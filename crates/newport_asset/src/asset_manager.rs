use crate::{
    Asset, 
    AssetCollection, 
    AssetRef, 
    AssetVariant, 

    engine::{ Module, Engine, EngineBuilder }, 
    log::{ info, error, Logger }
};

use std::{
    any::{ TypeId, Any },
    path::{ Path, PathBuf },
    sync::{ Arc, RwLock, RwLockReadGuard },
    time::{ SystemTime, Instant },
    ffi::OsStr,
    marker::PhantomData,
    fs,
    fmt,
};

#[derive(Debug)]
pub struct AssetEntry {
    pub path:       PathBuf,
    pub variant:    usize, // Index into AssetManager::variants
    pub write_time: SystemTime,

    // Arc which is used for asset ref counting. As long as there is more 
    // than 1 reference the asset is ensured to be loaded
    pub asset: Arc<RwLock<Option<Box<dyn Any>>>>,
}

pub struct AssetManagerInner {
    /// Variants and Collections are taken on initialized and do not change over the runtime of program
    pub variants:    Vec<AssetVariant>,
    pub collections: Vec<AssetCollection>,
    
    /// Assets are currently kept in a vector hidden behind a RwLock
    /// 
    /// # Todo
    ///
    /// * Faster asset lookup
    pub assets: RwLock<Vec<AssetEntry>>,
}

/// Manager that handles assets across multiple threads. 
#[derive(Clone)]
pub struct AssetManager(pub(crate) Arc<AssetManagerInner>);

impl AssetManager {
    pub fn assets(&self) -> RwLockReadGuard<Vec<AssetEntry>> {
        self.0.assets.read().unwrap()
    }

    /// Discovers all assets using the registered collection and variants
    pub fn discover(&self) {
        let collections = &self.0.collections;
        // Start off by making sure every collection exist
        for it in collections.iter() {
            if !it.path.exists() {
                fs::create_dir(&it.path).unwrap();
                info!("[AssetManager] Created collection directory {:?}", it.path);
            }
        }

        // Recursive file directory lookup. Sending members instead of AssetManager is due to RWLock on assets
        fn discover(mut path: PathBuf, assets: &mut Vec<AssetEntry>, variants: &Vec<AssetVariant>) -> PathBuf {
            // Iterate through every entry in a directory
            for entry in fs::read_dir(path.as_path()).unwrap() {
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
        
        let variants = &self.0.variants;

        // Run through each collection recursively discovering assets
        let mut assets_lock = self.0.assets.write().unwrap();
        for it in collections.iter() { 
            info!("[AssetManager] Discovering assets in {:?}", it.path);
            discover(it.path.clone(), &mut *assets_lock, &variants); 
        }
    }

    /// TODO
    pub fn find<P: AsRef<Path> + fmt::Debug, T: Asset>(&self, p: P) -> Option<AssetRef<T>> {
        let read_lock = self.0.assets.read().unwrap();
        let entry = read_lock.iter().find(|it| it.path == p.as_ref())?;

        // Assert if the type is incorrect
        let variant = &self.0.variants[entry.variant];
        assert!(TypeId::of::<T>() == variant.type_id);
        
        let result = AssetRef { 
            arc:     entry.asset.clone(), // Increment ref count
            phantom: PhantomData,
            variant: entry.variant,
            manager: self.clone(),
            path:    PathBuf::from(p.as_ref()),
        };

        // If we're the first reference the load the asset
        if result.strong_count() == 1 {
            let mut lock = entry.asset.write().unwrap();
            let (result, dur) = {
                let now = Instant::now();
                let result = (variant.load)(&entry.path);
                let dur = Instant::now().duration_since(now).as_secs_f64() * 1000.0;
                (result, dur)
            };
            if result.is_err() {
                let err = result.err().unwrap();
                error!("[AssetManager] Failed to load {:?} due to {:?}", p, err);
            } else {
                *lock = Some(result.unwrap());
                info!("[AssetManager] Loaded asset {:?} in {:.2}ms", p, dur);
            }
        }

        Some(result)
    }
}

impl AssetManager {
    pub fn new(variants: Vec<AssetVariant>, collections: Vec<AssetCollection>) -> Self {
        Self(Arc::new(AssetManagerInner{
            variants:    variants,
            collections: collections,
            assets:      RwLock::new(Vec::new()),
        }))
    }
}

impl Module for AssetManager {
    fn new() -> Self {
        let engine = Engine::as_ref();

        let variants = engine.register::<AssetVariant>().unwrap_or_default();
        let collections = engine.register::<AssetCollection>().unwrap_or_default();
        let result = AssetManager::new(variants, collections);
        result.discover();
        result
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Logger>()
            .register(AssetCollection::new("assets/"))
    }
}