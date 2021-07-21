#![feature(box_syntax)]
#![feature(trait_alias)]
#![feature(string_remove_matches)]
//! This crate provides a completely thread safe asset manager which
//! handles defining assets, loading assets, ref counting assets, and
//! serialization.

pub(crate) use newport_cache as cache;
pub(crate) use newport_engine as engine;
pub(crate) use newport_log as log;
pub(crate) use newport_serde as serde;

mod importer;
mod manager;
mod path_cache;
mod reference;
mod uuid;

pub use {
	importer::*,
	manager::*,
	reference::*,
	uuid::*,
};

pub(crate) use path_cache::*;

use std::{
	error,
	path::{
		Path,
		PathBuf,
	},
	result,
};

pub trait Asset: Sized + 'static {}

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

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
