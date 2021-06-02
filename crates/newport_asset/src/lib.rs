#![feature(box_syntax)]
#![feature(trait_alias)]
#![feature(string_remove_matches)]
//! This crate provides a completely thread safe asset manager which 
//! handles defining assets, loading assets, ref counting assets, and 
//! serialization.

pub(crate) use newport_engine as engine;
pub(crate) use newport_log    as log;
pub(crate) use newport_cache  as cache;
pub(crate) use newport_serde  as serde;

pub use std::path::{ Path, PathBuf };

pub use ron::de;
pub use ron::ser;

mod asset;
mod asset_manager;
mod asset_ref;
mod registers;
mod asset_cache;
mod uuid;

pub use {
    asset::*,
    asset_manager::*,
    asset_ref::*,
    registers::*,
    uuid::*,
};

pub(crate) use asset_cache::*;