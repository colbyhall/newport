use std::env;
use std::path::PathBuf;

fn main() {
	// copy the dll to the out dir
	let out_dir = env::var("OUT_DIR").unwrap();
	let mut path = PathBuf::from(out_dir);
	path.push("GFSDK_Aftermath_Lib.x64.dll");
	std::fs::copy("sdk/GFSDK_Aftermath_Lib.x64.dll", &path).unwrap();
}
