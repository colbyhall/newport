use crate::{
    cache,
    serde,
    engine,
    log::info,

    Asset,
    UUID,
    AssetCollection,
    AssetVariant
};

use cache::Cache;

use serde::{
    Serialize,
    Deserialize,
    Deserializer
};

use engine::Engine;

use std::{
    collections::HashMap,
    path::PathBuf,
    fs
};

#[derive(Serialize)]
#[serde(crate = "self::serde")]
pub struct AssetFile<T: Asset> {
    pub id: UUID,
    pub asset: T,
}

impl<'de, T: Asset> Deserialize<'de> for AssetFile<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de> {
        Deserialize::deserialize(deserializer)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct AssetCache {
    uuid_to_path: HashMap<UUID, PathBuf>,
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
                info!("Created collection directory {:?}", it.path);
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
                            let contents = fs::read_to_string(&path).unwrap();
                            let uuid = (variant.deserialize_uuid)(&contents);

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
            info!("Discovering assets in {:?}", it.path);
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
