pub mod sys;
use sys::*;

use std::fs::File;
use std::io::Write;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::slice;
use std::time::SystemTime;

pub type Result<T> = std::result::Result<T, GFSDK_Aftermath_Result>;

static DMPS_PATH: &str = "target/logs/";

unsafe extern "C" fn gpu_crash_dump_callback(
	dump: *const c_void,
	dump_size: size_t,
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

pub fn enable_gpu_crash_dumps() -> Result<()> {
	let result = unsafe {
		GFSDK_Aftermath_EnableGpuCrashDumps(
			GFSDK_Aftermath_Version_GFSDK_Aftermath_Version_API,
			0,
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

pub fn disable_gpu_crash_dumps() -> Result<()> {
	let result = unsafe { GFSDK_Aftermath_DisableGpuCrashDumps() };
	if result == GFSDK_Aftermath_Result_GFSDK_Aftermath_Result_Success {
		Ok(())
	} else {
		Err(result)
	}
}
