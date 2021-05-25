use hassle_rs::{
    Dxc, 
    DxcCompiler, 
    DxcLibrary,
    DxcIncludeHandler
};

use std::{
    env,
    path,
};

use path::PathBuf;

use crate::ShaderVariant;

// This is copied from utils.rs in hassle-rs
struct DefaultIncludeHandler {}

impl DxcIncludeHandler for DefaultIncludeHandler {
    fn load_source(&self, filename: String) -> Option<String> {
        use std::io::Read;
        match std::fs::File::open(filename) {
            Ok(mut f) => {
                let mut content = String::new();
                f.read_to_string(&mut content).unwrap();
                Some(content)
            }
            Err(_) => None,
        }
    }
}

struct CompilerThreadInfo {
    _dxc: Dxc,
    compiler: DxcCompiler,
    library: DxcLibrary
}

impl CompilerThreadInfo {
    fn new() -> Self {
        let out_dir = env!("OUT_DIR");
        let target_index = out_dir.find("target").unwrap();
        let (_, relative_out_dir) = out_dir.split_at(target_index);
        
        let mut library_path = PathBuf::from(relative_out_dir);
        library_path.push("dxcompiler.dll");

        let dxc = Dxc::new(Some(library_path)).unwrap();

        let compiler = dxc.create_compiler().unwrap();
        let library = dxc.create_library().unwrap();

        Self{
            _dxc: dxc,
            compiler,
            library,
        }
    }
}

thread_local! {
    static DXC_COMPILER: CompilerThreadInfo = CompilerThreadInfo::new();
}

pub fn compile(name: &str, source: &str, main: &str, variant: ShaderVariant) -> Option<Vec<u8>> {  
    DXC_COMPILER.with(|f| {
        let target_profile = match variant {
            ShaderVariant::Pixel => "ps_6_1",
            ShaderVariant::Vertex => "vs_6_1"
        };
    
        let blob = f.library
            .create_blob_with_encoding_from_str(source)
            .unwrap();

        let mut args = Vec::with_capacity(4); // TODO: Temp allocator

        #[cfg(feature = "vulkan")]
        {
            args.push("-spirv");
            args.push("-Zpc"); // Column major matrices

            if variant == ShaderVariant::Vertex {
                args.push("-fvk-invert-y");
            }
        }

        let result = f.compiler.compile(
            &blob,
            name,
            main,
            target_profile,
            &args[..],
            Some(Box::new(DefaultIncludeHandler {})),
            &[],
        );

        match result {
            Err(_) => {
                // let error_blob = result
                //     .0
                //     .get_error_buffer()
                //     .unwrap();
                // TODO: Do error handling
                None
            }
            Ok(result) => {
                let result_blob = result.get_result().unwrap();
                Some(result_blob.to_vec())
            }
        }
    })
}