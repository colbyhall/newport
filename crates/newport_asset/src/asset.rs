use crate::{
    serde,
    UUID,
    deserialize
};

use serde::{
    Serialize, 
    de::DeserializeOwned,
};

use std::path::Path;

pub trait Asset: Sized + 'static {
    fn load(bytes: &[u8], path: &Path) -> (UUID, Self);
}

impl<T: Serialize + DeserializeOwned + Sized + 'static> Asset for T {
    fn load(bytes: &[u8], _path: &Path) -> (UUID, Self) {
        deserialize(bytes).expect("Failed to deserialize asset")
    }
}