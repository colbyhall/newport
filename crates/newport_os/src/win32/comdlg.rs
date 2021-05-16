use crate::win32::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct OPENFILENAMEA {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: LPCSTR,
    pub lpstrCustomFilter: LPSTR,
    pub nMaxCustFilter: DWORD,
    pub nFilterIndex: DWORD,
    pub lpstrFile: LPSTR,
    pub nMaxFile: DWORD,
    pub lpstrFileTitle: LPSTR,
    pub nMaxFileTitle: DWORD,
    pub lpstrInitialDir: LPCSTR,
    pub lpstrTitle: LPCSTR,
    pub Flags: DWORD,
    pub nFileOffset: WORD,
    pub nFileExtension: WORD,
    pub lpstrDefExt: LPCSTR,
    pub lCustData: LPARAM,
    pub lpfnHook: PVOID,
    pub lpTemplateName: LPCSTR,
    pub lpEditInfo: PVOID,
    pub lpstrPrompt: LPCSTR,
    pub pvReserved: PVOID,
    pub dwReserved: DWORD,
    pub FlagsEx: DWORD,
}

pub const OFN_PATHMUSTEXIST: DWORD = 0x00000800;
pub const OFN_FILEMUSTEXIST: DWORD = 0x00001000;

#[link(name = "comdlg32")]
extern "stdcall" {
    pub fn GetOpenFileNameA(param1: *mut OPENFILENAMEA) -> BOOL;
}