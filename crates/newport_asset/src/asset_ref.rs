use crate::{
    AssetManager,
    log::info,
};

use std::{
    sync::{ Arc, RwLock, RwLockReadGuard, RwLockWriteGuard },
    marker::PhantomData,
    any::Any,
    path::{ PathBuf, Path },
    ops::{ Deref, DerefMut },
    fmt,
};

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
    pub(crate) arc:     Arc<RwLock<Option<Box<dyn Any>>>>,
    pub(crate) phantom: PhantomData<T>,
    pub(crate) variant: usize, // Index into AssetManager::variants
    pub(crate) manager: AssetManager, 
    pub(crate) path:    PathBuf,
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

    pub fn path(&self) -> &Path {
        &self.path
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
            let variant = &self.manager.0.variants[self.variant];

            let mut lock = self.arc.write().unwrap();
            *lock = None;
            info!("[AssetManager] Unloaded asset {:?}", self.path)
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
            path:       self.path.clone(),
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