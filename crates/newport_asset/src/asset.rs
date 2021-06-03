use crate::{
    serde,
    UUID,
    deserialize
};

use serde::{
    Serialize, 
    de::DeserializeOwned,
};

pub trait Asset: Sized + 'static {
    fn load(bytes: &[u8]) -> (UUID, Self);
}

impl<T: Serialize + DeserializeOwned + Sized + 'static> Asset for T {
    fn load(bytes: &[u8]) -> (UUID, Self) {
        deserialize(bytes).expect("Failed to deserialize asset")
    }
}