use super::*;

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct Oem_ID {
	wProcessorArchitecture: WORD,
	wReserved: WORD,
}

#[repr(C)]
union Oem_Union {
	dwOemId: DWORD,
	internal: Oem_ID,
}

#[repr(C)]
struct SYSTEM_INFO {
	oem_id: Oem_Union,
	dwPageSize: DWORD,
	lpMinimumApplicationAddress: LPVOID,
	lpMaximumApplicationAddress: LPVOID,
	dwActiveProcessorMask: DWORD_PTR,
	dwNumberOfProcessors: DWORD,
	dwProcessorType: DWORD,
	dwAllocationGranularity: DWORD,
	wProcessorLevel: WORD,
	wProcessorRevision: WORD,
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
	Level: BYTE,
	Associativity: BYTE,
	LineSize: WORD,
	Size: DWORD,
	Type: PROCESSOR_CACHE_TYPE,
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
	NodeNumber: DWORD,
	Cache: CACHE_DESCRIPTOR,
	Reserved: [ULONGLONG; 2],
}

impl Default for SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION {
	fn default() -> Self {
		SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION {
			ProcessorCoreFlags: 0,
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SYSTEM_LOGICAL_PROCESSOR_INFORMATION {
	ProcessorMask: ULONG_PTR,
	Relationship: LOGICAL_PROCESSOR_RELATIONSHIP,
	Union: SYSTEM_LOGICAL_PROCESSOR_INFORMATION_UNION,
}
type PSYSTEM_LOGICAL_PROCESSOR_INFORMATION = *mut SYSTEM_LOGICAL_PROCESSOR_INFORMATION;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SYSTEMTIME {
	pub wYear: WORD,
	pub wMonth: WORD,
	pub wDayOfWeek: WORD,
	pub wDay: WORD,
	pub wHour: WORD,
	pub wMinute: WORD,
	pub wSecond: WORD,
	pub wMilliseconds: WORD,
}
pub type LPSYSTEMTIME = *mut SYSTEMTIME;

pub const MEM_COMMIT: DWORD = 0x00001000;
pub const MEM_RESERVE: DWORD = 0x00002000;
pub const MEM_RELEASE: DWORD = 0x00008000;

pub const PAGE_READWRITE: DWORD = 0x04;

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
	fn GetLogicalProcessorInformation(
		Buffer: PSYSTEM_LOGICAL_PROCESSOR_INFORMATION,
		ReturnedLength: PDWORD,
	) -> BOOL;

	pub fn GetLastError() -> DWORD;

	pub fn GetLocalTime(lpSystemTime: LPSYSTEMTIME);

	fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
	fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
	fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
	fn GetProcessHeap() -> HANDLE;

	fn VirtualAlloc(
		lpAddress: LPVOID,
		dwSize: SIZE_T,
		flAlloactionType: DWORD,
		flProtect: DWORD,
	) -> LPVOID;
	fn VirtualFree(lpAddress: LPVOID, dwSize: SIZE_T, dwFreeType: DWORD) -> BOOL;
}
