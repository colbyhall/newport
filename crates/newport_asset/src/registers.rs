use crate::{
    LoadError,
    Asset
};

use std::{
    any::{ Any, TypeId },
    path::{ Path, PathBuf },
};

#[derive(Clone)]
pub struct AssetVariant {
    pub(crate) type_id:    TypeId,
    pub(crate) extension:  &'static str,
    
    pub(crate) load:     fn(&Path) -> Result<Box<dyn Any>, LoadError>,
    pub(crate) unload:   fn(Box<dyn Any>)
}

impl AssetVariant {
    pub fn new<T: Asset>() -> AssetVariant {
        fn load<T: Asset>(path: &Path) -> Result<Box<dyn Any>, LoadError> {
            let t = T::load(path)?;
            Ok(Box::new(t))
        }

        fn unload<T: Asset>(asset: Box<dyn Any>) {
            let t = asset.downcast::<T>().unwrap();
            T::unload(*t);
        }

        AssetVariant{
            type_id:    TypeId::of::<T>(),

            extension:  T::extension(),
            load:       load::<T>,
            unload:     unload::<T>,
        }
    }
}

#[derive(Clone)]
pub struct AssetCollection {
    pub(crate) path: PathBuf,
}

impl AssetCollection {
    pub fn new(path: impl Into<PathBuf>) -> AssetCollection {
        AssetCollection{
            path: path.into(),
        }
    }
}