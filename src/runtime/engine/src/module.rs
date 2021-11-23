use crate::Builder;

/// Modules are an easy way to have global immutable state
pub trait Module: Sized + 'static {
	/// Creates a module and returns as result. This is the initialization point for Modules
	fn new() -> Self;

	/// Takes a builder to append on other modules or elements
	///
	/// # Arguments
	///
	/// * `builder` - A [`Builder`] used to add dep modules or functions
	fn depends_on(builder: Builder) -> Builder {
		builder
	}
}

#[macro_export]
macro_rules! define_run_module {
	($module:ident, $name:literal) => {
		fn main() -> std::io::Result<()> {
			$crate::Builder::new().module::<$module>().name($name).run()
		}
	};
}
