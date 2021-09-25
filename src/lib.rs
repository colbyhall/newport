// Runnable structure
pub mod engine {
	pub use newport_engine::*;
}

// Completely thread safe asset manager
pub mod asset {
	pub use newport_asset::*;
}

// GPU abstraction
pub mod gpu {
	pub use newport_gpu::*;
}

// Runtime graphics using the gpu crate
pub mod graphics {
	pub use newport_graphics::*;
}

// No std math library that works for shaders
pub mod math {
	pub use newport_math::*;
}

pub mod platform {
	pub use newport_platform::*;
}

pub mod serde {
	pub use newport_serde::*;
}

pub mod game {
	pub use newport_game::*;
}
