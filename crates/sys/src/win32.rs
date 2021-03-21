#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]

pub type PVOID   = *mut u8;
pub type LPVOID  = PVOID;
pub type LPCVOID = *const u8;

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

// @NOTE(colby): Window Styles
pub const WS_OVERLAPPED     : u32 = 0x00000000;
pub const WS_POPUP          : u32 = 0x80000000;
pub const WS_CHILD          : u32 = 0x40000000;
pub const WS_MINIMIZE       : u32 = 0x20000000;
pub const WS_VISIBLE        : u32 = 0x10000000;
pub const WS_DISABLED       : u32 = 0x08000000;
pub const WS_CLIPSIBLINGS   : u32 = 0x04000000;
pub const WS_CLIPCHILDREN   : u32 = 0x02000000;
pub const WS_MAXIMIZE       : u32 = 0x01000000;
pub const WS_CAPTION        : u32 = 0x00C00000;
pub const WS_BORDER         : u32 = 0x00800000;
pub const WS_DLGFRAME       : u32 = 0x00400000;
pub const WS_VSCROLL        : u32 = 0x00200000;
pub const WS_HSCROLL        : u32 = 0x00100000;
pub const WS_SYSMENU        : u32 = 0x00080000;
pub const WS_THICKFRAME     : u32 = 0x00040000;
pub const WS_GROUP          : u32 = 0x00020000;
pub const WS_TABSTOP        : u32 = 0x00010000;

pub const WS_MINIMIZEBOX : u32 = 0x00020000;
pub const WS_MAXIMIZEBOX : u32 = 0x00010000;

pub const WP_OVERLAPPEDWINDOW : u32 = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;

pub const GWLP_USERDATA : i32 = -21;

pub const PM_NOREMOVE : u32 = 0x0000;
pub const PM_REMOVE   : u32 = 0x0001;
pub const PM_NOYIELD  : u32 = 0x0002;

// Window Messages
pub const WM_DESTROY   : u32 = 0x0002;
pub const WM_SIZE      : u32 = 0x0005;
pub const WM_SETFOCUS  : u32 = 0x0007;
pub const WM_KILLFOCUS : u32 = 0x0008;
pub const WM_CLOSE     : u32 = 0x0010;
pub const WM_QUIT      : u32 = 0x0012;

pub const WM_KEYDOWN : u32 = 0x0100;
pub const WM_KEYUP   : u32 = 0x0101;
pub const WM_CHAR    : u32 = 0x0102;

pub const WM_SYSKEYDOWN : u32 = 0x0104;
pub const WM_SYSKEYUP   : u32 = 0x0105;
pub const WM_SYSCHAR    : u32 = 0x0106;

pub const WM_MOUSEMOVE      : u32 = 0x0200;
pub const WM_LBUTTONDOWN    : u32 = 0x0201;
pub const WM_LBUTTONUP      : u32 = 0x0202;
pub const WM_LBUTTONDBLCLK  : u32 = 0x0203;
pub const WM_RBUTTONDOWN    : u32 = 0x0204;
pub const WM_RBUTTONUP      : u32 = 0x0205;
pub const WM_RBUTTONDBLCLK  : u32 = 0x0206;
pub const WM_MBUTTONDOWN    : u32 = 0x0207;
pub const WM_MBUTTONUP      : u32 = 0x0208;
pub const WM_MBUTTONDBLCLK  : u32 = 0x0209;
pub const WM_MOUSEWHEEL     : u32 = 0x020A;
pub const WM_XBUTTONDOWN    : u32 = 0x020B;
pub const WM_XBUTTONUP      : u32 = 0x020C;
pub const WM_XBUTTONDBLCLK  : u32 = 0x020D;
pub const WM_MOUSEHWHEEL    : u32 = 0x020E;

pub const WM_SIZING : u32 = 0x0214;

pub const WM_SYSCOMMAND : u32 = 0x0112;
pub const SC_MAXIMIZE   : u64 = 0xF030;

pub const SM_CXSCREEN   : i32 = 0;
pub const SM_CYSCREEN   : i32 = 1;

pub const SW_HIDE   : i32 = 0;
pub const SW_SHOW   : i32 = 5;

pub const SWP_NOSIZE    : u32 = 0x0001;
pub const SWP_NOZORDER  : u32 = 0x0004;

pub const MEM_COMMIT    : DWORD = 0x00001000;
pub const MEM_RESERVE   : DWORD = 0x00002000;
pub const MEM_RELEASE   : DWORD = 0x00008000;

pub const PAGE_READWRITE: DWORD = 0x04;

pub const MONITOR_DEFAULTTONULL     : DWORD = 0x00000000;
pub const MONITOR_DEFAULTTOPRIMARY  : DWORD = 0x00000001;
pub const MONITOR_DEFAULTTONEAREST  : DWORD = 0x00000002;

#[macro_export]
macro_rules! HIWORD {
    ($_dw:expr) => (((($_dw as ULONG_PTR) >> 16) & 0xFFFF) as WORD)
}

const WHEEL_DELTA : u32 = 120;

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
#[derive(Copy, Clone, Default)]
pub struct RECT {
    pub left:   LONG,
    pub top:    LONG,
    pub right:  LONG,
    pub bottom: LONG,
}
pub type LPRECT = *mut RECT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

pub type WNDPROC = extern fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WNDCLASSEXA {
    pub cbSize:        UINT,
    pub style:         UINT,
    pub lpfnWndProc:   Option<WNDPROC>,
    pub cbClsExtra:    u32,
    pub cbWndExtra:    u32,
    pub hInstance:     HINSTANCE,
    pub hIcon:         HICON,
    pub hCursor:       HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName:  LPCSTR,
    pub lpszClassName: LPCSTR,
    pub hIconSm:       HICON,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MSG {
    pub hWnd:       HWND,
    pub message:    UINT,
    pub wParam:     WPARAM,
    pub lParam:     LPARAM,
    pub time:       DWORD,
    pub pt:         POINT,
}
pub type LPMSG = *mut MSG;

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct Oem_ID {
    wProcessorArchitecture: WORD,
    wReserved:              WORD,
}

#[repr(C)]
union Oem_Union {
    dwOemId:  DWORD,
    internal: Oem_ID,
}

#[repr(C)]
struct SYSTEM_INFO {
    oem_id:                      Oem_Union,
    dwPageSize:                  DWORD,
    lpMinimumApplicationAddress: LPVOID,
    lpMaximumApplicationAddress: LPVOID,
    dwActiveProcessorMask:       DWORD_PTR,
    dwNumberOfProcessors:        DWORD,
    dwProcessorType:             DWORD,
    dwAllocationGranularity:     DWORD,
    wProcessorLevel:             WORD,
    wProcessorRevision:          WORD,
}
type LPSYSTEM_INFO = *mut SYSTEM_INFO;

#[repr(C)]
#[derive(Copy, Clone)]
enum PROCESSOR_CACHE_TYPE {
    CacheUnified,
    CacheInstruction,
    CacheData,
    CacheTrace,
}

impl Default for PROCESSOR_CACHE_TYPE {
    fn default() -> Self {
        PROCESSOR_CACHE_TYPE::CacheUnified
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct CACHE_DESCRIPTOR {
    Level:          BYTE,
    Associativity:  BYTE,
    LineSize:       WORD,
    Size:           DWORD,
    Type:           PROCESSOR_CACHE_TYPE,
}

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

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
enum LOGICAL_PROCESSOR_RELATIONSHIP {
    RelationProcessorCore,
    RelationNumaNode,
    RelationCache,
    RelationProcessorPackage,
    RelationGroup,
    RelationAll,
}

impl Default for LOGICAL_PROCESSOR_RELATIONSHIP {
    fn default() -> Self {
        LOGICAL_PROCESSOR_RELATIONSHIP::RelationProcessorCore
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
union SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION {
    ProcessorCoreFlags: BYTE,
    NodeNumber:         DWORD,
    Cache:              CACHE_DESCRIPTOR,
    Reserved:           [ULONGLONG; 2],
}

impl Default for SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION {
    fn default() -> Self {
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION { ProcessorCoreFlags: 0, }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
    ProcessorMask: ULONG_PTR,
    Relationship:  LOGICAL_PROCESSOR_RELATIONSHIP,
    Union:         SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION,
}
type PSYSTEM_LOGICAL_PROCESSOR_INFORMATION = *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SYSTEMTIME {
    wYear:          WORD,
    wMonth:         WORD,
    wDayOfWeek:     WORD,
    wDay:           WORD,
    wHour:          WORD,
    wMinute:        WORD,
    wSecond:        WORD,
    wMilliseconds:  WORD,
}
type LPSYSTEMTIME = *mut SYSTEMTIME;

#[link(name = "kernel32")]
extern "stdcall" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetModuleHandleA(lpModuleName: LPCSTR) -> HMODULE;
    
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> LPVOID;
    pub fn LoadLibraryA(lpLibFileName: LPCSTR) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;

    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;

    fn GetSystemInfo(lpSystemInfo: LPSYSTEM_INFO);
    fn GetLogicalProcessorInformation(Buffer: PSYSTEM_LOGICAL_PROCESSOR_INFORMATION, ReturnedLength: PDWORD) -> BOOL;

    fn GetLastError() -> DWORD;

    fn GetLocalTime(lpSystemTime: LPSYSTEMTIME);

    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;

    fn VirtualAlloc(lpAddress: LPVOID, dwSize: SIZE_T, flAlloactionType: DWORD, flProtect: DWORD) -> LPVOID;
    fn VirtualFree(lpAddress: LPVOID, dwSize: SIZE_T, dwFreeType: DWORD) -> BOOL;
}

#[link(name = "user32")]
extern "stdcall" {
    pub fn GetCaretBlinkTime() -> UINT;

    pub fn RegisterClassExA(Arg1: *const WNDCLASSEXA) -> ATOM;
    pub fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: u32, Y: u32, nWidth: u32, nHeight: u32, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: i32, Y: i32, cx: i32, cy: i32, uFlags: UINT) -> BOOL;

    pub fn AdjustWindowRect(lpRect: LPRECT, dwStyle: DWORD, bMenu: BOOL) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;

    pub fn SetWindowLongPtrA(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR;
    pub fn GetWindowLongPtrA(hWnd: HWND, nIndex: i32) -> LONG_PTR;

    pub fn PeekMessageA(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT) -> BOOL;
    pub fn GetMessageA(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageA(lpMsg: *const MSG) -> LRESULT;

    pub fn LoadCursorA(hInstance: HINSTANCE, lpCursorName: LPCSTR) -> HCURSOR;

    pub fn GetSystemMetrics(nIndex: i32) -> i32;

    pub fn SetCapture(hWnd: HWND) -> HWND;
    pub fn ReleaseCapture() -> BOOL;

    pub fn MonitorFromWindow(hwnd: HWND, dwFlags: DWORD) -> HMONITOR;
}