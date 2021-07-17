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

mod asset;
mod asset_cache;
mod asset_manager;
mod asset_ref;
mod de;
mod registers;
mod uuid;

pub use {asset::*, asset_cache::*, asset_manager::*, asset_ref::*, de::*, registers::*, uuid::*};
