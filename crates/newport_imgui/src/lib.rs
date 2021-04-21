#![feature(trait_alias)]

pub(crate) use newport_math as math;
pub(crate) use newport_graphics as graphics;
pub(crate) use newport_gpu as gpu;
pub(crate) use newport_asset as asset;
pub(crate) use newport_os as os;
pub(crate) use newport_engine as engine;

mod builder;
mod context;
mod id;
mod retained;
mod paint;
mod layout;
mod input;
mod widgets;

pub use builder::*;
pub use context::*;
pub use id::*;
pub use retained::*;
pub use paint::*;
pub use layout::*;
pub use input::*;
pub use widgets::*;