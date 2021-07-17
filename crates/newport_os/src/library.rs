use std::{
    ffi::{c_void, CStr, CString},
    ptr::copy_nonoverlapping,
};

#[macro_export]
macro_rules! proc_address {
    ($lib:expr, $proc:expr) => {{
        let func_ptr = $lib.raw_address($proc);
        if func_ptr.is_null() {
            None
        } else {
            Some(unsafe { std::mem::transmute(func_ptr) })
        }
    }};
}

pub struct Library {
    handle: *mut c_void,
}

impl Library {
    #[cfg(windows)]
    pub fn new(path: &str) -> Result<Library, ()> {
        let path = CString::new(path.as_bytes()).unwrap();

        unsafe {
            let handle = LoadLibraryA(path.as_ptr() as LPCSTR);
            if handle == INVALID_HANDLE_VALUE {
                Err(())
            } else {
                Ok(Library {
                    handle: handle as *mut c_void,
                })
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub fn new(path: &str) -> Result<Library, ()> {
        use libc::{dlopen, RTLD_LAZY, RTLD_LOCAL};

        let cstr = CString::new(path.as_bytes()).unwrap();
        unsafe {
            let handle = dlopen(cstr.as_ptr(), RTLD_LOCAL | RTLD_LAZY);
            if handle.is_null() {
                Err(()) // () makes my life easier
            } else {
                Ok(Library { handle })
            }
        }
    }

    #[cfg(windows)]
    fn proc_addr(&self, buf: &[u8]) -> *const c_void {
        use crate::win32::GetProcAddress;

        let cstr = CString::new(path.as_bytes()).unwrap();
        unsafe { GetProcAddress(self.handle, buf.as_ptr() as LPCSTR) as *const c_void }
    }

    #[cfg(target_os = "linux")]
    fn proc_addr(&self, buf: &[u8]) -> *const c_void {
        use libc::dlsym;

        unsafe { dlsym(self.handle, buf.as_ptr() as *mut i8) }
    }

    pub fn raw_address(&self, proc: &str) -> *const c_void {
        assert!(proc.len() < 512);
        let mut buf = [0 as u8; 512];
        unsafe { copy_nonoverlapping(proc.as_ptr(), buf.as_mut_ptr(), proc.len()) };
        buf[proc.len()] = 0;

        self.proc_addr(&buf)
    }
}

impl Drop for Library {
    #[cfg(windows)]
    fn drop(&mut self) {
        use crate::win32::HMODULE;

        unsafe {
            FreeLibrary(self.handle as HMODULE);
        }
    }

    #[cfg(target_os = "linux")]
    fn drop(&mut self) {
        use libc::dlclose;

        unsafe { dlclose(self.handle) };
    }
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}
