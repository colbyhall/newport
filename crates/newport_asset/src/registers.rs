use crate::{
    Asset,
    UUID,
};

use std::{
    any::{ Any, TypeId },
    path::{ Path, PathBuf },
};

#[derive(Clone)]
pub struct AssetVariant {
    pub(crate) type_id:    TypeId,
    pub(crate) extensions: Vec<&'static str>,

    pub(crate) deserialize: fn(&[u8], &Path) -> (UUID, Box<dyn Any>),
}

impl AssetVariant {
    pub fn new<T: Asset>(extensions: &[&'static str]) -> AssetVariant {
        fn deserialize<T: Asset>(contents: &[u8], path: &Path) -> (UUID, Box<dyn Any>) {
            let (id, t) = T::load(contents, path);
            (id, Box::new(t))
        }

        AssetVariant{
            type_id:    TypeId::of::<T>(),
            extensions: extensions.to_vec(),

            deserialize: deserialize::<T>,
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

    pub fn path(&self) -> &Path {
        &self.path
    }
}