#[cfg(target_os = "windows")]
#[path = "win32.rs"]
mod internal;

#[cfg(target_os = "windows")]
pub use internal::*;