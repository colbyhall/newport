#![feature(const_type_name)]

use {
	engine::{
		Engine,
		Module,
	},
	serde::{
		de::DeserializeOwned,
		toml::Value,
		Serialize,
	},
	std::{
		any::{
			Any,
			TypeId,
		},
		collections::HashMap,
		fs,
		path::Path,
		path::PathBuf,
	},
};

pub struct ConfigManager {
	pub(crate) entries: HashMap<TypeId, Box<dyn Any>>,
}

impl ConfigManager {
	pub fn read<'a, T: Config>() -> &'a T {
		let manager: &ConfigManager = Engine::module().unwrap();
		let id = TypeId::of::<T>();

		let entry = manager.entries.get(&id).unwrap_or_else(|| {
			panic!(
				"Config of type \"{}\" is not registered.",
				std::any::type_name::<T>()
			)
		});

		entry.downcast_ref().unwrap()
	}
}

impl Module for ConfigManager {
	fn new() -> Self {
		let mut registers: Vec<ConfigVariant> = Engine::register().to_vec();
		if registers.is_empty() {
			return Self {
				entries: HashMap::default(),
			};
		}

		// Create the config directory if one does not exist
		if !Path::new(CONFIG_PATH).exists() {
			fs::create_dir(CONFIG_PATH).unwrap();
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
		let mut entries: HashMap<TypeId, Box<dyn Any>> =
			registers.drain(..).map(|r| (r.id, (r.default)())).collect();

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
						*entry = (deserialize)(value.clone()); // FIXME: This could be super slow. Should do a replace or something
					} else {
						// TODO: This really should be logging
					}
				}
			} else {
				// Create a file with the default config text
				fs::write(&path, DEFAULT_CONFIG_FILE).unwrap();
			}
		}

		Self { entries }
	}
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
pub const INPUT_CONFIG_FILE: &str = "input.toml";

pub trait Config: Serialize + DeserializeOwned + 'static + Default {
	const NAME: &'static str = std::any::type_name::<Self>();
	const FILE: &'static str;

	fn variant() -> ConfigVariant {
		let id = TypeId::of::<Self>();
		let name = Self::NAME;
		let name = {
			let camel_case = name.rsplit_once("::").unwrap_or(("", name)).1;

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
			file: Self::FILE,
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
