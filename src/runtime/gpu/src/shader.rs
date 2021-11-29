use hassle_rs::{
	Dxc,
	DxcCompiler,
	DxcIncludeHandler,
	DxcLibrary,
};

use std::sync::Arc;
use std::{
	env,
	path,
};

use path::PathBuf;

use crate::{
	api,
	Device,
	Gpu,
	Result,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ShaderVariant {
	Vertex,
	Pixel,
}

pub struct ShaderBuilder<'a> {
	binary: &'a [u8],
	variant: ShaderVariant,
	main: &'static str,

	device: Option<&'a Device>,
}

impl<'a> ShaderBuilder<'a> {
	pub fn main(mut self, main: &'static str) -> Self {
		self.main = main;
		self
	}

	pub fn device(mut self, device: &'a Device) -> Self {
		self.device = Some(device);
		self
	}

	pub fn spawn(self) -> Result<Shader> {
		let device = match self.device {
			Some(device) => device,
			None => Gpu::device(),
		};

		Ok(Shader(api::Shader::new(
			device.0.clone(),
			self.binary,
			self.variant,
			self.main.to_string(),
		)?))
	}
}

#[derive(Clone)]
pub struct Shader(pub(crate) Arc<api::Shader>);

impl Shader {
	pub fn builder(binary: &'_ [u8], variant: ShaderVariant) -> ShaderBuilder<'_> {
		ShaderBuilder {
			binary,
			variant,
			main: "main",

			device: None,
		}
	}
}

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
	library: DxcLibrary,
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

		Self {
			_dxc: dxc,
			compiler,
			library,
		}
	}
}

thread_local! {
	static DXC_COMPILER: CompilerThreadInfo = CompilerThreadInfo::new();
}

pub fn compile(
	name: &str,
	source: &str,
	main: &str,
	variant: ShaderVariant,
) -> std::result::Result<Vec<u8>, String> {
	DXC_COMPILER.with(|f| {
		let target_profile = match variant {
			ShaderVariant::Pixel => "ps_6_1",
			ShaderVariant::Vertex => "vs_6_1",
		};

		let blob = f
			.library
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
			Err(result) => {
				let error_blob = result.0.get_error_buffer().unwrap();

				let err_string = f
					.library
					.get_blob_as_string(&error_blob)
					.replace("\\n", "\n");
				Err(err_string)
			}
			Ok(result) => {
				let result_blob = result.get_result().unwrap();
				Ok(result_blob.to_vec())
			}
		}
	})
}
