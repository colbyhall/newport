use crate::{
    serde,
    
    Asset,
    AssetFile,
    UUID,
};

use serde::ron;

use std::{
    any::{ Any, TypeId },
    path::{ Path, PathBuf },
};

#[derive(Clone)]
pub struct AssetVariant {
    pub(crate) type_id:    TypeId,
    pub(crate) extensions: Vec<&'static str>,

    pub(crate) deserialize: fn(&str) -> Box<dyn Any>,
    pub(crate) deserialize_uuid: fn(&str) -> UUID,
}

impl AssetVariant {
    pub fn new<T: Asset>(extensions: &[&'static str]) -> AssetVariant {
        fn deserialize<T: Asset>(contents: &str) -> Box<dyn Any> {
            let mut t: AssetFile<T> = ron::from_str(contents).expect("Failed to deserialize asset");
            t.asset.post_load();

            Box::new(t)
        }

        fn deserialize_uuid<T: Asset>(contents: &str) -> UUID {
            let t: AssetFile<T> = ron::from_str(contents).expect("Failed to deserialize asset");

            t.id
        }

        AssetVariant{
            type_id:    TypeId::of::<T>(),
            extensions: extensions.to_vec(),

            deserialize: deserialize::<T>,
            deserialize_uuid: deserialize_uuid::<T>,
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