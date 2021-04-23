#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]

use std::ffi::c_void;

pub type PVOID   = *mut c_void;
pub type LPVOID  = PVOID;
pub type LPCVOID = *const c_void;

pub type BOOL = u32;
pub type CHAR = u8;
pub type INT  = u32;
pub type UINT = u32;

pub type HANDLE     = PVOID;
pub type HINSTANCE  = HANDLE;
pub type HMODULE    = HANDLE;
pub type HWND       = HANDLE;
pub type HGLOBAL    = HANDLE;
pub type HICON      = HANDLE;
pub type HCURSOR    = HICON;
pub type HBRUSH     = HANDLE;
pub type HMENU      = HANDLE;
pub type HMONITOR   = HANDLE;

pub type LONG  = u32;
pub type PLONG = *mut LONG;

pub type LONGLONG  = i64;
pub type ULONGLONG = u64;

pub type LPSTR  = *mut i8;
pub type LPCSTR = *const i8;

pub type LPWSTR  = *mut i16;
pub type LPCWSTR = *const i16;

pub type WORD    = u16;
pub type DWORD   = u32;
pub type LPDWORD = *mut DWORD;
pub type PDWORD  = LPDWORD;

pub type ULONG_PTR = u64;
pub type LONG_PTR  = i64;
pub type UINT_PTR  = u64;
pub type DWORD_PTR = DWORD;

pub type FLOAT  = f32;
pub type INT64  = i64;
pub type INT32  = u32;
pub type USHORT = u16;

pub type BYTE   = u8;
pub type SIZE_T = usize;

pub type HRESULT = LONG;
pub type LRESULT = LONG_PTR;
pub type WPARAM  = UINT_PTR;
pub type LPARAM  = LONG_PTR;

type ATOM    = WORD;

#[macro_export]
macro_rules! MAKEINTRESOURCEA {
    ($i:expr) => ((($i as WORD) as LONG_PTR) as LPSTR)
}

#[allow(overflowing_literals)]
pub const INFINITE     : u32 = 0xFFFFFFFF;
pub const GMEM_MOVABLE : u32 = 0x02;
pub const CF_TEXT      : u32 = 1;

pub const INVALID_HANDLE_VALUE : HANDLE = (-1 as LONG_PTR) as HANDLE;

#[macro_export]
macro_rules! HIWORD {
    ($_dw:expr) => (((($_dw as ULONG_PTR) >> 16) & 0xFFFF) as WORD)
}

pub const WHEEL_DELTA : u32 = 120;

#[macro_export]
macro_rules! GET_WHEEL_DELTA_WPARAM {
    ($wParam:expr) => (HIWORD!($wParam) as i16)
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LARGE_INTEGER_SPLIT {
    pub LowPart:  DWORD,
    pub HighPart: LONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union LARGE_INTEGER {
    pub u:          LARGE_INTEGER_SPLIT,
    pub QuadPart:   LONGLONG,
}
pub type PLARGE_INTEGER = *mut LARGE_INTEGER;


#[repr(C)]
#[derive(Copy, Clone)]
pub struct ULARGE_INTEGER_SPLIT {
    pub LowPart:  DWORD,
    pub HighPart: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ULARGE_INTEGER {
    pub u:          ULARGE_INTEGER_SPLIT,
    pub QuadPart:   ULONGLONG,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct RECT {
    pub left:   LONG,
    pub top:    LONG,
    pub right:  LONG,
    pub bottom: LONG,
}
pub type LPRECT = *mut RECT;

impl RECT {
    pub fn point_in_rect(&self, p: POINT) -> bool {
        (self.left <= p.x && p.x < self.right) && (self.top <= p.y && p.y < self.bottom)
    }
}   

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}
pub type LPPOINT = *mut POINT;

pub mod user32;
pub use user32::*;

pub mod kernel32;
pub use kernel32::*;

pub mod dwmapi;
pub use dwmapi::*;

pub mod uxtheme;
pub use uxtheme::*;