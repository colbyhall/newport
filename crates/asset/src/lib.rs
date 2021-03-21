//! This crate provides an asset manager which handles defining
//! assets, loading assets, ref counting assets, and serialization 

pub struct AssetManager {

}

static mut ASSET_MANAGER: Option<AssetManager> = None;

impl AssetManager {
    pub fn init() {
        unsafe {
            ASSET_MANAGER = Some(AssetManager{

            });
        }
    }
}