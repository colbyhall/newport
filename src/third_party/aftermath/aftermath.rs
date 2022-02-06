pub mod sys;
use libloading::Library;
use sys::*;

use std::fs::File;
use std::io::Write;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::slice;
use std::time::SystemTime;

pub type Result<T> = std::result::Result<T, GFSDK_Aftermath_Result>;

static DMPS_PATH: &str = "target/gpu_dmps/";

unsafe extern "C" fn gpu_crash_dump_callback(
	dump: *const c_void,
	dump_size: u32,
	_user_data: *mut c_void,
) {
	// dump to file
	let mut path = PathBuf::new();
	path.push(DMPS_PATH);
	path.push(format!(
		"game_{}.nv-gpudmp",
		SystemTime::UNIX_EPOCH
			.duration_since(SystemTime::now())
			.unwrap()
			.as_nanos()
	));

	let mut file = File::create(path).unwrap();
	file.write_all(slice::from_raw_parts(dump as *const u8, dump_size as usize))
		.unwrap();
}

pub struct Aftermath {
	library: Library,
}

impl Aftermath {
	pub fn new() -> std::result::Result<Aftermath, libloading::Error> {
		let out_dir = env!("OUT_DIR");
		let target_index = out_dir.find("target").unwrap();
		let (_, relative_out_dir) = out_dir.split_at(target_index);

		let mut library_path = PathBuf::from(relative_out_dir);
		library_path.push("GFSDK_Aftermath_Lib.x64.dll");

		let library = unsafe { Library::new(library_path)? };

		Ok(Aftermath { library })
	}

	pub fn enable_gpu_crash_dumps(&self) -> Result<()> {
		let enable = unsafe {
			self.library
				.get::<PFN_GFSDK_Aftermath_EnableGpuCrashDumps>(
					b"GFSDK_Aftermath_EnableGpuCrashDumps",
				)
				.unwrap()
				.lift_option()
				.unwrap()
		};
		let result = unsafe {
			enable(
				GFSDK_Aftermath_Version_GFSDK_Aftermath_Version_API,
				GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags_GFSDK_Aftermath_GpuCrashDumpWatchedApiFlags_Vulkan,
				GFSDK_Aftermath_GpuCrashDumpFeatureFlags_GFSDK_Aftermath_GpuCrashDumpFeatureFlags_DeferDebugInfoCallbacks,
				Some(gpu_crash_dump_callback),
				None,
				None,
				std::ptr::null_mut(),
			)
		};
		if result == GFSDK_Aftermath_Result_GFSDK_Aftermath_Result_Success {
			Ok(())
		} else {
			Err(result)
		}
	}

	pub fn disable_gpu_crash_dumps(&self) -> Result<()> {
		let disable = unsafe {
			self.library
				.get::<PFN_GFSDK_Aftermath_DisableGpuCrashDumps>(
					b"GFSDK_Aftermath_DisableGpuCrashDumps",
				)
				.unwrap()
				.lift_option()
				.unwrap()
		};
		let result = unsafe { disable() };
		if result == GFSDK_Aftermath_Result_GFSDK_Aftermath_Result_Success {
			Ok(())
		} else {
			Err(result)
		}
	}
}
