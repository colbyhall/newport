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

type HRESULT = LONG;
type LRESULT = LONG_PTR;
type WPARAM  = UINT_PTR;
type LPARAM  = LONG_PTR;

type ATOM    = WORD;

macro_rules! MAKEINTRESOURCEA {
    ($i:expr) => ((($i as WORD) as LONG_PTR) as LPSTR)
}

#[allow(overflowing_literals)]
const INFINITE     : u32 = 0xFFFFFFFF;
const GMEM_MOVABLE : u32 = 0x02;
const CF_TEXT      : u32 = 1;

pub const INVALID_HANDLE_VALUE : HANDLE = (-1 as LONG_PTR) as HANDLE;

// @NOTE(colby): Window Styles
const WS_OVERLAPPED     : u32 = 0x00000000;
const WS_POPUP          : u32 = 0x80000000;
const WS_CHILD          : u32 = 0x40000000;
const WS_MINIMIZE       : u32 = 0x20000000;
const WS_VISIBLE        : u32 = 0x10000000;
const WS_DISABLED       : u32 = 0x08000000;
const WS_CLIPSIBLINGS   : u32 = 0x04000000;
const WS_CLIPCHILDREN   : u32 = 0x02000000;
const WS_MAXIMIZE       : u32 = 0x01000000;
const WS_CAPTION        : u32 = 0x00C00000;
const WS_BORDER         : u32 = 0x00800000;
const WS_DLGFRAME       : u32 = 0x00400000;
const WS_VSCROLL        : u32 = 0x00200000;
const WS_HSCROLL        : u32 = 0x00100000;
const WS_SYSMENU        : u32 = 0x00080000;
const WS_THICKFRAME     : u32 = 0x00040000;
const WS_GROUP          : u32 = 0x00020000;
const WS_TABSTOP        : u32 = 0x00010000;

const WS_MINIMIZEBOX : u32 = 0x00020000;
const WS_MAXIMIZEBOX : u32 = 0x00010000;

const WP_OVERLAPPEDWINDOW : u32 = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;

const GWLP_USERDATA : i32 = -21;

const PM_NOREMOVE : u32 = 0x0000;
const PM_REMOVE   : u32 = 0x0001;
const PM_NOYIELD  : u32 = 0x0002;

// @NOTE(colby): Window Messages
const WM_DESTROY   : u32 = 0x0002;
const WM_SIZE      : u32 = 0x0005;
const WM_SETFOCUS  : u32 = 0x0007;
const WM_KILLFOCUS : u32 = 0x0008;
const WM_CLOSE     : u32 = 0x0010;
const WM_QUIT      : u32 = 0x0012;

const WM_KEYDOWN : u32 = 0x0100;
const WM_KEYUP   : u32 = 0x0101;
const WM_CHAR    : u32 = 0x0102;

const WM_SYSKEYDOWN : u32 = 0x0104;
const WM_SYSKEYUP   : u32 = 0x0105;
const WM_SYSCHAR    : u32 = 0x0106;

const WM_MOUSEMOVE      : u32 = 0x0200;
const WM_LBUTTONDOWN    : u32 = 0x0201;
const WM_LBUTTONUP      : u32 = 0x0202;
const WM_LBUTTONDBLCLK  : u32 = 0x0203;
const WM_RBUTTONDOWN    : u32 = 0x0204;
const WM_RBUTTONUP      : u32 = 0x0205;
const WM_RBUTTONDBLCLK  : u32 = 0x0206;
const WM_MBUTTONDOWN    : u32 = 0x0207;
const WM_MBUTTONUP      : u32 = 0x0208;
const WM_MBUTTONDBLCLK  : u32 = 0x0209;
const WM_MOUSEWHEEL     : u32 = 0x020A;
const WM_XBUTTONDOWN    : u32 = 0x020B;
const WM_XBUTTONUP      : u32 = 0x020C;
const WM_XBUTTONDBLCLK  : u32 = 0x020D;
const WM_MOUSEHWHEEL    : u32 = 0x020E;

const WM_SIZING : u32 = 0x0214;

const WM_SYSCOMMAND : u32 = 0x0112;
const SC_MAXIMIZE   : u64 = 0xF030;

const SM_CXSCREEN   : i32 = 0;
const SM_CYSCREEN   : i32 = 1;

const SW_HIDE   : i32 = 0;
const SW_SHOW   : i32 = 5;

const SWP_NOSIZE    : u32 = 0x0001;
const SWP_NOZORDER  : u32 = 0x0004;

const MEM_COMMIT    : DWORD = 0x00001000;
const MEM_RESERVE   : DWORD = 0x00002000;
const MEM_RELEASE   : DWORD = 0x00008000;

const PAGE_READWRITE: DWORD = 0x04;

const MONITOR_DEFAULTTONULL     : DWORD = 0x00000000;
const MONITOR_DEFAULTTOPRIMARY  : DWORD = 0x00000001;
const MONITOR_DEFAULTTONEAREST  : DWORD = 0x00000002;

macro_rules! HIWORD {
    ($_dw:expr) => (((($_dw as ULONG_PTR) >> 16) & 0xFFFF) as WORD)
}

const WHEEL_DELTA : u32 = 120;
macro_rules! GET_WHEEL_DELTA_WPARAM {
    ($wParam:expr) => (HIWORD!($wParam) as i16)
}

#[repr(C)]
#[derive(Copy, Clone)]
struct LARGE_INTEGER_SPLIT {
    LowPart:  DWORD,
    HighPart: LONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
union LARGE_INTEGER {
    u:          LARGE_INTEGER_SPLIT,
    QuadPart:   LONGLONG,
}
type PLARGE_INTEGER = *mut LARGE_INTEGER;


#[repr(C)]
#[derive(Copy, Clone)]
struct ULARGE_INTEGER_SPLIT {
    LowPart:  DWORD,
    HighPart: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
union ULARGE_INTEGER {
    u:          ULARGE_INTEGER_SPLIT,
    QuadPart:   ULONGLONG,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct RECT {
    left:   LONG,
    top:    LONG,
    right:  LONG,
    bottom: LONG,
}
type LPRECT = *mut RECT;

#[repr(C)]
#[derive(Copy, Clone)]
struct POINT {
    x: LONG,
    y: LONG,
}

type WNDPROC = extern fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
#[derive(Copy, Clone)]
struct WNDCLASSEXA {
    cbSize:        UINT,
    style:         UINT,
    lpfnWndProc:   Option<WNDPROC>,
    cbClsExtra:    u32,
    cbWndExtra:    u32,
    hInstance:     HINSTANCE,
    hIcon:         HICON,
    hCursor:       HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName:  LPCSTR,
    lpszClassName: LPCSTR,
    hIconSm:       HICON,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MSG {
    hWnd:       HWND,
    message:    UINT,
    wParam:     WPARAM,
    lParam:     LPARAM,
    time:       DWORD,
    pt:         POINT,
}
type LPMSG = *mut MSG;

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
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn GetModuleHandleA(lpModuleName: LPCSTR) -> HMODULE;
    
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> LPVOID;
    fn LoadLibraryA(lpLibFileName: LPCSTR) -> HMODULE;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;

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

    fn RegisterClassExA(Arg1: *const WNDCLASSEXA) -> ATOM;
    fn CreateWindowExA(dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: u32, Y: u32, nWidth: u32, nHeight: u32, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn DestroyWindow(hWnd: HWND) -> BOOL;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: i32, Y: i32, cx: i32, cy: i32, uFlags: UINT) -> BOOL;

    fn AdjustWindowRect(lpRect: LPRECT, dwStyle: DWORD, bMenu: BOOL) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;

    fn SetWindowLongPtrA(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetWindowLongPtrA(hWnd: HWND, nIndex: i32) -> LONG_PTR;

    fn PeekMessageA(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT) -> BOOL;
    fn GetMessageA(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageA(lpMsg: *const MSG) -> LRESULT;

    fn LoadCursorA(hInstance: HINSTANCE, lpCursorName: LPCSTR) -> HCURSOR;

    fn GetSystemMetrics(nIndex: i32) -> i32;

    fn SetCapture(hWnd: HWND) -> HWND;
    fn ReleaseCapture() -> BOOL;

    fn MonitorFromWindow(hwnd: HWND, dwFlags: DWORD) -> HMONITOR;
}