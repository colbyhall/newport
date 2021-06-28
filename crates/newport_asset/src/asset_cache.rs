use crate::{
    cache,
    serde,
    engine,
    log::info,

    UUID,
    AssetCollection,
    AssetVariant
};

use cache::Cache;

use serde::{
    Serialize,
    Deserialize,
};

use engine::Engine;

use std::{
    collections::HashMap,
    path::PathBuf,
    fs
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "self::serde")]
pub struct AssetCache {
    pub uuid_to_path: HashMap<UUID, PathBuf>,
}

impl Cache for AssetCache {
    fn new() -> Self {
        let engine = Engine::as_ref();

        let collections = engine.register::<AssetCollection>().unwrap_or_default();
        let variants = engine.register::<AssetVariant>().unwrap_or_default();
        
        // Run through all the collections and create a directory if one is not created
        for it in collections.iter() {
            if !it.path.exists() {
                fs::create_dir(&it.path).unwrap();
                info!("Created collection directory ({})", it.path.display());
            }
        }

        fn discover(mut path: PathBuf, uuid_to_path: &mut HashMap<UUID, PathBuf>, variants: &Vec<AssetVariant>) -> PathBuf {
            for entry in fs::read_dir(&path).unwrap() {
                let entry = entry.unwrap();
                let file_type = entry.file_type().unwrap();

                if file_type.is_dir() {
                    path.push(entry.file_name());
                    path = discover(path, uuid_to_path, variants);
                    path.pop();
                } else if file_type.is_file() {
                    let path = entry.path();
                    let ext = path.extension().unwrap_or_default();

                    let variant = variants.iter().find(|v| v.extensions.contains(&ext.to_str().unwrap()));
                    match variant {
                        Some(variant) => {
                            let contents = fs::read(&path).unwrap();
                            let uuid = (variant.deserialize)(&contents, &path).0;

                            uuid_to_path.insert(uuid, path);
                        },
                        _ => {}
                    }
                } else {
                    continue;
                }
            }

            path
        }
        
        let mut uuid_to_path = HashMap::new();
        for it in collections.iter() {
            info!("Discovering assets in ({})", it.path.display());
            discover(it.path.clone(), &mut uuid_to_path, &variants);
        }

        Self {
            uuid_to_path
        }
    }

    fn needs_reload(&self) -> bool {
        false
    }
}
