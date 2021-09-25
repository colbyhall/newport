use std::any::Any;
use std::collections::HashMap;

use asset::{
	Asset,
	Importer,
};

use super::{
	Entity,
	EntityInfo,
};

use engine::Engine;
use engine::Uuid;

use serde::{
	ron::{
		self,
		Value,
	},
	Deserialize,
	Serialize,
};

use super::ComponentVariant;
use super::ComponentVariantId;

#[derive(Debug)]
pub struct SceneEntry {
	pub id: Uuid,
	pub components: HashMap<ComponentVariantId, Box<dyn Any>>,
}

#[derive(Debug)]
pub struct Scene {
	pub entities: Vec<SceneEntry>,
}

impl Asset for Scene {}

#[derive(Serialize, Deserialize)]
pub(crate) struct SceneImporter {}

impl Importer for SceneImporter {
	type Target = Scene;

	fn import(&self, bytes: &[u8]) -> asset::Result<Self::Target> {
		let id_key = Value::String("id".to_string());
		let components_key = Value::String("components".to_string());

		let variants: HashMap<String, ComponentVariant> = Engine::register::<ComponentVariant>()
			.unwrap()
			.iter()
			.map(|it| (it.name.to_string(), it.clone()))
			.collect();

		let value: Value = ron::from_str(std::str::from_utf8(bytes)?)?;
		match value {
			Value::Seq(seq) => {
				let mut entries = Vec::with_capacity(seq.len());
				for it in seq.iter() {
					match it {
						Value::Map(map) => {
							if map.len() != 2 {
								todo!()
							}

							let id = map[&id_key].clone();
							let id: Uuid = id.into_rust()?;

							let components = &map[&components_key];
							match components {
								Value::Map(map) => {
									let mut components = HashMap::with_capacity(map.len());
									for (key, value) in map.iter() {
										let name = match key {
											Value::String(s) => s,
											_ => todo!(),
										};

										let variant = variants.get(name).unwrap_or_else(|| todo!());
										let component = (variant.parse_value)(value.clone())?;

										components.insert(variant.id, component);
									}
									entries.push(SceneEntry { id, components });
								}
								_ => todo!(),
							}
						}
						_ => todo!(),
					}
				}
				Ok(Scene { entities: entries })
			}
			_ => todo!(),
		}
	}
}

#[derive(Default, Clone)]
pub struct SceneRuntime {
	pub entities: HashMap<Entity, EntityInfo>,
}
