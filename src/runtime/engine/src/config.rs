use std::{
	any::{
		Any,
		TypeId,
	},
	collections::HashMap,
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
	file: &'static str,
	name: String,
	pub(crate) value: Box<dyn Any>,

	deserialize: fn(Value) -> Box<dyn Any>,
	// default: fn() -> Box<dyn Any>,
}

pub struct ConfigMap {
	pub(crate) entries: HashMap<TypeId, Entry>,
}

const CONFIG_PATH: &str = "config/";

impl ConfigMap {
	pub(crate) fn new(mut registers: Vec<ConfigRegister>) -> Self {
		if registers.is_empty() {
			return Self {
				entries: HashMap::default(),
			};
		}

		if !Path::new(CONFIG_PATH).exists() {
			create_dir(CONFIG_PATH).unwrap();
		}

		let mut entries: HashMap<TypeId, Entry> = registers
			.drain(..)
			.map(|r| {
				(
					r.id,
					Entry {
						file: r.file,
						name: r.name,

						value: (r.default)(),

						deserialize: r.deserialize,
						// default: r.default,
					},
				)
			})
			.collect();

		for entry in fs::read_dir(CONFIG_PATH).unwrap() {
			let entry = entry.unwrap();
			let file_type = entry.file_type().unwrap();

			if file_type.is_file() {
				let path = entry.path();

				let stem = path.file_stem().unwrap().to_str().unwrap().to_string();

				let file: Value = fs::read_to_string(path).unwrap().parse().unwrap();
				let table = file.as_table().unwrap();
				for (name, value) in table.iter() {
					let entry = entries
						.iter_mut()
						.find(|(_, v)| &v.name == name && v.file == stem)
						.unwrap_or_else(|| panic!("Unregisted config structure \"{}\"", name))
						.1;
					let deserialize = entry.deserialize;
					entry.value = (deserialize)(value.clone());
				}
			}
		}

		Self { entries }
	}
}

pub trait Config: Serialize + DeserializeOwned + 'static + Default {}

#[derive(Clone)]
pub struct ConfigRegister {
	file: &'static str,
	name: String,
	id: TypeId,

	deserialize: fn(Value) -> Box<dyn Any>,
	default: fn() -> Box<dyn Any>,
}

impl ConfigRegister {
	pub fn new<T: Config>(file: &'static str) -> Self {
		let id = TypeId::of::<T>();
		let name = {
			let camel_case = std::any::type_name::<T>()
				.rsplit_once("::")
				.unwrap_or(("", std::any::type_name::<T>()))
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

		Self {
			file,
			name,
			id,
			deserialize: my_deserialize::<T>,
			default: my_default::<T>,
		}
	}
}
