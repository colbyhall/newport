// Fundamental types used in all newport packages
pub mod core {
    pub use newport_core::*;
}

// Core runnable structure 
pub mod engine {
    pub use newport_engine::*;
}

// Global thread safe logger
pub mod log {
    pub use newport_log::*;
}

// Completely thread safe asset manager 
pub mod asset {
    pub use newport_asset::*;
}

// GPU abstraction
pub mod gpu {
    pub use newport_gpu::*;
}