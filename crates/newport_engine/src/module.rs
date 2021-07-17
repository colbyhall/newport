use crate::EngineBuilder;

/// Modules are an easy way to have global immutable state
pub trait Module: Sized + 'static {
    /// Creates a module and returns as result. This is the initialization point for Modules
    fn new() -> Self;

    /// Takes a builder to append on other modules or elements
    ///
    /// # Arguments
    ///
    /// * `builder` - A [`EngineBuilder`] used to add dep modules or functions
    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
    }
}
