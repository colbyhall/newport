use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
	path::PathBuf,
};

use serde::{
	de::DeserializeOwned,
	Serialize,
};

use std::fs;
use std::fs::create_dir;
use std::path::Path;

use toml::Value;

pub(crate) struct Entry {
	pub(crate) value: Box<dyn Any>,

	deserialize: fn(Value) -> Box<dyn Any>,
	// default: fn() -> Box<dyn Any>,
}

pub struct ConfigMap {
	pub(crate) entries: HashMap<TypeId, Entry>,
}

const CONFIG_PATH: &str = "config/";

const DEFAULT_CONFIG_FILE: &str = "# Example Config File
#
# [game]
# default_scene = \"{44EF625A-017F-4FD9-8AEB-AF41C8585C05}\"
#
# [game.data]
# health = 100.0
# name = \"Billy Bob\"
# enemies = [ 0, 123, 351243 ]
";

// TODO: DIFFING
// const USER_CONFIG_PATH: &str = "target/config/";

pub const ENGINE_CONFIG_FILE: &str = "engine.toml";

impl ConfigMap {
	pub(crate) fn new(mut registers: Vec<ConfigVariant>) -> Self {
		if registers.is_empty() {
			return Self {
				entries: HashMap::default(),
			};
		}

		// Create the config directory if one does not exist
		if !Path::new(CONFIG_PATH).exists() {
			create_dir(CONFIG_PATH).unwrap();
		}

		// Seperate config to their file
		let mut files_to_variants: HashMap<&'static str, HashMap<String, ConfigVariant>> =
			HashMap::with_capacity(32);
		registers.iter().for_each(|e| {
			if let Some(variants) = files_to_variants.get_mut(&e.file) {
				variants.insert(e.name.clone(), e.clone());
			} else {
				let mut variants = HashMap::with_capacity(32);
				variants.insert(e.name.clone(), e.clone());
				files_to_variants.insert(e.file, variants);
			}
		});

		// Create an entry for every registered config type
		let mut entries: HashMap<TypeId, Entry> = registers
			.drain(..)
			.map(|r| {
				(
					r.id,
					Entry {
						value: (r.default)(),
						deserialize: r.deserialize,
						// default: r.default,
					},
				)
			})
			.collect();

		// Iterate through all file paths. Open each file and iterate through
		// the table members. Deserialize into `entries`
		for (file, variants) in files_to_variants.iter() {
			let mut path = PathBuf::from(CONFIG_PATH);
			path.push(file);

			if let Ok(file) = fs::read_to_string(&path) {
				let file: Value = file.parse().unwrap();
				let table = file.as_table().unwrap();
				for (name, value) in table.iter() {
					if let Some(variant) = variants.get(name) {
						let deserialize = variant.deserialize;

						let entry = entries.get_mut(&variant.id).unwrap();
						entry.value = (deserialize)(value.clone()); // @TODO: This could be super slow. Should do a replace or something
					} else {
						// TODO: This really should be logging
					}
				}
			} else {
				// Create an empty file if none existed
				// @TODO: make default file spawn with example text
				fs::write(&path, DEFAULT_CONFIG_FILE).unwrap();
			}
		}

		Self { entries }
	}
}

pub trait Config: Serialize + DeserializeOwned + 'static + Default {
	fn variant(file: &'static str) -> ConfigVariant {
		let id = TypeId::of::<Self>();
		let name = {
			let camel_case = std::any::type_name::<Self>()
				.rsplit_once("::")
				.unwrap_or(("", std::any::type_name::<Self>()))
				.1;

			let mut snake_case = String::with_capacity(camel_case.len() * 2);
			for (index, c) in camel_case.chars().enumerate() {
				assert!(c.is_alphabetic());

				if c.is_uppercase() {
					if index > 0 {
						snake_case.push('_');
					}
					snake_case.push_str(&c.to_lowercase().to_string());
				} else {
					snake_case.push(c);
				}
			}
			snake_case
		};

		fn my_default<T: Config>() -> Box<dyn Any> {
			Box::new(T::default())
		}

		fn my_deserialize<T: Config>(value: Value) -> Box<dyn Any> {
			let value: T = value.try_into().unwrap();
			Box::new(value)
		}

		ConfigVariant {
			file,
			name,
			id,
			deserialize: my_deserialize::<Self>,
			default: my_default::<Self>,
		}
	}
}

#[derive(Clone)]
pub struct ConfigVariant {
	file: &'static str,
	name: String,
	id: TypeId,

	deserialize: fn(Value) -> Box<dyn Any>,
	default: fn() -> Box<dyn Any>,
}
