use std::path::Path;

/// Trait alis for what an `Asset` can be
pub trait Asset: Sized + 'static {
    fn load(path: &Path) -> Result<Self, LoadError>;
    fn unload(_asset: Self) { }
    fn extension() -> &'static str;
}

/// Enum for asset load errors
#[derive(Debug)]
pub enum LoadError {
    FileNotFound,
    ParseError,
    DataError
}