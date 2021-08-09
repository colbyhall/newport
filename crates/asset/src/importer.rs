use std::marker::PhantomData;

use serde::{
	self,
	de::DeserializeOwned,
	ron,
	Deserialize,
	Serialize,
};

use crate::{
	Asset,
	Result,
	UUID,
};

use std::{
	any::{
		Any,
		TypeId,
	},
	str,
};

pub(crate) const META_EXTENSION: &str = ".meta";

pub trait Importer: Sized + Serialize + DeserializeOwned + 'static {
	type Target: Asset;

	fn import(&self, bytes: &[u8]) -> Result<Self::Target>;
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct NativeImporter<T: Asset> {
	phantom: PhantomData<T>,
}

impl<T: Asset + Serialize + DeserializeOwned> Importer for NativeImporter<T> {
	type Target = T;

	fn import(&self, bytes: &[u8]) -> Result<Self::Target> {
		let contents = str::from_utf8(bytes)?;
		Ok(ron::from_str(contents)?)
	}
}

#[derive(Clone)]
pub struct Variant {
	pub(crate) asset: TypeId,
	pub(crate) importer: TypeId,

	pub(crate) extensions: Vec<&'static str>,

	pub(crate) load_asset: fn(&Box<dyn Any>, &[u8]) -> Result<Box<dyn Any>>,
	pub(crate) load_meta: fn(&[u8]) -> Result<(UUID, Box<dyn Any>)>,
}

impl Variant {
	pub fn new<T: Importer>(extensions: &[&'static str]) -> Variant {
		fn load_asset<T: Importer>(meta: &Box<dyn Any>, bytes: &[u8]) -> Result<Box<dyn Any>> {
			let meta = meta.downcast_ref::<T>().unwrap();
			Ok(Box::new(meta.import(bytes)?))
		}

		fn load_meta<T: Importer>(bytes: &[u8]) -> Result<(UUID, Box<dyn Any>)> {
			#[derive(Serialize, Deserialize)]
			#[serde(crate = "self::serde", rename = "Meta")]
			struct MetaFile<T> {
				uuid: UUID,
				#[serde(bound(deserialize = "T: Deserialize<'de>"))]
				importer: T,
			}

			let contents = str::from_utf8(bytes)?;
			let meta: MetaFile<T> = ron::from_str(contents)?;
			Ok((meta.uuid, Box::new(meta.importer)))
		}

		Variant {
			asset: TypeId::of::<T::Target>(),
			importer: TypeId::of::<T>(),

			extensions: extensions.to_vec(),

			load_asset: load_asset::<T>,
			load_meta: load_meta::<T>,
		}
	}
}
