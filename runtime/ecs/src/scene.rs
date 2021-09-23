use std::collections::HashMap;
use std::panic::PanicInfo;

use asset::{
	Asset,
	Importer,
};

use engine::info;
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

use crate::ComponentVariant;

pub struct Scene;

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
			.map(|it| (it.name.to_string(), it))
			.collect();

		let value: Value = ron::from_str(std::str::from_utf8(bytes)?)?;
		match value {
			Value::Seq(seq) => {
				for it in seq.iter() {
					match it {
						Value::Map(map) => {
							if map.len() != 2 {
								todo!()
							}

							let id = map[&id_key].clone();
							let components = &map[&components_key];

							let id: Uuid = id.into_rust()?;
							info!("{:?}", id);

							match components {
								Value::Map(map) => {
									for (key, value) in map.iter() {
										let name = match key {
											Value::String(s) => s,
											_ => todo!(),
										};

										let variant = variants.get(name).unwrap_or_else(|| todo!());

										info!("{:?}", it);
									}
								}
								_ => todo!(),
							}
						}
						_ => todo!(),
					}
				}
			}
			_ => todo!(),
		}
		// info!("{:?}", value);
		Ok(Scene)
	}
}
