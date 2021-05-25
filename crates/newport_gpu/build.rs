use std::{
    fs,
    env,
    path::PathBuf
};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut path = PathBuf::from(out_dir);
    path.push("dxcompiler.dll");
    fs::copy("bin/dxcompiler.dll", &path).unwrap();
}