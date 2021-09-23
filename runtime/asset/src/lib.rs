#![feature(box_syntax)]
#![feature(trait_alias)]
//! This crate provides a completely thread safe asset manager which
//! handles defining assets, loading assets, ref counting assets, and
//! serialization.

mod importer;
mod manager;
mod path_cache;
mod reference;

pub(crate) use engine::Uuid;

pub use {
	importer::*,
	manager::*,
	reference::*,
};

pub(crate) use path_cache::*;

use std::{
	path::{
		Path,
		PathBuf,
	},
	result,
};

use engine::define_log_category;

define_log_category!(AssetManager, ASSET_MANAGER_CATEGORY);

pub trait Asset: Sized + 'static {
	fn default_uuid() -> Option<Uuid> {
		None
	}
}

pub type Result<T> = result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub struct Collection {
	pub(crate) path: PathBuf,
}

impl Collection {
	pub fn new(path: impl Into<PathBuf>) -> Collection {
		Collection { path: path.into() }
	}

	pub fn path(&self) -> &Path {
		&self.path
	}
}
