use std::env;
use std::path::{
	Path,
	PathBuf,
};

fn main() {
	let base = Path::new(file!()).parent().unwrap();
	let mut path = PathBuf::from(base);
	path.push("sdk/GFSDK_Aftermath_Lib.x64");

	// copy the dll to the out dir
	// let out_dir = env::var("OUT_DIR").unwrap();
	// let mut path = PathBuf::from(out_dir);
	// path.push("GFSDK_Aftermath_Lib.x64.dll");
	// std::fs::copy("sdk/GFSDK_Aftermath_Lib.x64.dll", &path).unwrap();

	// Tell cargo to tell rustc to link to the aftermath lib
	// shared library.
	println!("cargo:rustc-link-lib={}", path.display());

	// Search for the dll in the out dir
	println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());

	// Tell cargo to invalidate the built crate whenever the wrapper changes
	println!("cargo:rerun-if-changed=sdk/GFSDK_Aftermath_Wrapper.hpp");

	// The bindgen::Builder is the main entry point
	// to bindgen, and lets you build up options for
	// the resulting bindings.
	let bindings = bindgen::Builder::default()
		// The input header we would like to generate
		// bindings for.
		.header("sdk/GFSDK_Aftermath_Wrapper.hpp")
		// Tell cargo to invalidate the built crate whenever any of the
		// included header files changed.
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		// Finish the builder and generate the bindings.
		.generate()
		// Unwrap the Result and panic on failure.
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/bindings.rs file.
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings!");
}
