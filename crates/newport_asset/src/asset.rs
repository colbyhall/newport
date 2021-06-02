use crate::{
    serde
};

use serde::{
    Serialize, 
    de::DeserializeOwned,
};

pub trait Asset: Serialize + DeserializeOwned + Sized + 'static {
    fn post_load(&mut self);
}

impl<T: Serialize + DeserializeOwned + Sized + 'static> Asset for T {
    fn post_load(&mut self) {
        // Do nothing
    }
}
