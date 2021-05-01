#![feature(box_syntax)]
#![feature(trait_alias)]
//! This crate provides a completely thread safe asset manager which 
//! handles defining assets, loading assets, ref counting assets, and 
//! serialization.


pub(crate) use newport_engine as engine;
pub(crate) use newport_log as log;

pub use std::path::{ Path, PathBuf };

pub use ron::de;
pub use ron::ser;

mod asset;
mod asset_manager;
mod asset_ref;
mod registers;

pub use asset::*;
pub use asset_manager::*;
pub use asset_ref::*;
pub use registers::*;