#[cfg(target_os = "windows")]
use crate::win32::*;

use std::ffi::CString;
use std::ptr::copy_nonoverlapping;

#[macro_export]
macro_rules! proc_address {
    ($lib:expr, $proc:expr) => (
        {
            let func_ptr = $lib.raw_address($proc);
            if func_ptr.is_null() {
                None
            } else {
                Some(unsafe { std::mem::transmute(func_ptr) })
            }
        }
    )
}

pub struct Library {
    #[cfg(target_os = "windows")]
    handle: HMODULE,
}

impl Library {
    pub fn new(path: &str) -> Result<Library, ()> {
        let path = CString::new(path.as_bytes()).unwrap(); // TODO: Temp allocator
        unsafe {
            let handle = LoadLibraryA(path.as_ptr() as LPCSTR);
            if handle == INVALID_HANDLE_VALUE {
                return Err(());
            }
            Ok(Library {
                handle: handle
            })
        }
    }

    pub fn raw_address(&self, proc: &str) -> LPVOID {
        assert!(proc.len() < 512);
        let mut buffer = [0 as u8; 512];
        unsafe { copy_nonoverlapping(proc.as_ptr(), buffer.as_mut_ptr(), proc.len()) };
        buffer[proc.len()] = 0;

        unsafe { GetProcAddress(self.handle, buffer.as_ptr() as LPCSTR) as _ }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.handle); }
    }
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}