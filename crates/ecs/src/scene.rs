use std::panic::PanicInfo;

use asset::{
	Asset,
	Importer,
};

use engine::info;

use serde::{
	ron::{
		self,
		Value,
	},
	Deserialize,
	Serialize,
};

pub struct Scene;

impl Asset for Scene {}

#[derive(Serialize, Deserialize)]
pub(crate) struct SceneImporter {}

impl Importer for SceneImporter {
	type Target = Scene;

	fn import(&self, bytes: &[u8]) -> asset::Result<Self::Target> {
		let id_key = Value::String("id".to_string());
		let components_key = Value::String("components".to_string());

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
