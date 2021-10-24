use crate::{
	Collection,
	Uuid,
	Variant,
	ASSET_MANAGER_CATEGORY,
};

use cache::Cache;

use serde::{
	self,
	Deserialize,
	Serialize,
};

use engine::{
	info,
	Engine,
};

use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PathCache {
	pub uuid_to_path: HashMap<Uuid, PathBuf>,
}

impl Cache for PathCache {
	fn new() -> Self {
		let collections = Engine::register::<Collection>().unwrap();
		let variants = Engine::register::<Variant>().unwrap();

		// Run through all the collections and create a directory if one is not created
		for it in collections.iter() {
			if !it.path.exists() {
				fs::create_dir(&it.path).unwrap();
				info!(
					ASSET_MANAGER_CATEGORY,
					"Created collection directory ({})",
					it.path.display()
				);
			}
		}

		fn discover(
			mut path: PathBuf,
			uuid_to_path: &mut HashMap<Uuid, PathBuf>,
			variants: &[Variant],
		) -> PathBuf {
			for entry in fs::read_dir(&path).unwrap() {
				let entry = entry.unwrap();
				let file_type = entry.file_type().unwrap();

				if file_type.is_dir() {
					path.push(entry.file_name());
					path = discover(path, uuid_to_path, variants);
					path.pop();
				} else if file_type.is_file() {
					let path = entry.path();

					let variant = {
						let ext = path.extension().unwrap_or_default();

						variants
							.iter()
							.find(|v| v.extensions.contains(&ext.to_str().unwrap()))
					};
					if let Some(variant) = variant {
						let mut meta_path = path.clone().into_os_string();
						meta_path.push(crate::META_EXTENSION);

						let contents = match fs::read(&meta_path) {
							Ok(contents) => contents,
							_ => continue,
						};
						info!(ASSET_MANAGER_CATEGORY, "Caching asset ({})", path.display());
						let uuid = (variant.load_meta)(&contents).unwrap().0;

						uuid_to_path.insert(uuid, path);
					}
				} else {
					continue;
				}
			}

			path
		}

		let mut uuid_to_path = HashMap::new();
		for it in collections.iter() {
			info!(
				ASSET_MANAGER_CATEGORY,
				"Discovering assets in ({})",
				it.path.display()
			);
			discover(it.path.clone(), &mut uuid_to_path, variants);
		}

		Self { uuid_to_path }
	}

	fn needs_reload(&self) -> bool {
		false
	}
}
