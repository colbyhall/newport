mod input;
pub use input::*;

pub mod time;

#[cfg(target_os = "windows")]
pub mod windows {
	pub use winapi::*;
}

#[cfg(target_os = "windows")]
use std::{
	ffi::OsStr,
	os::windows::ffi::OsStrExt,
};

#[cfg(target_os = "windows")]
use windows::{
	shared::{
		minwindef::{
			HMODULE,
			UINT,
		},
		windef::HWND,
	},
	um::{
		libloaderapi::{
			GetModuleHandleA,
			LoadLibraryW,
		},
		winuser,
	},
};

pub enum LibraryError {
	LibraryNotFound,
	SymbolNotFound,
}

#[cfg(target_os = "windows")]
pub struct Library {
	hmodule: HMODULE,
}

impl Library {
	pub fn new(path: impl AsRef<OsStr>) -> Result<Self, LibraryError> {
		unsafe {
			let path: Vec<u16> = path.as_ref().encode_wide().collect();
			let hmodule = LoadLibraryW(path.as_ptr());
			if hmodule.is_null() {
				Err(LibraryError::LibraryNotFound)
			} else {
				Ok(Self { hmodule })
			}
		}
	}
}

impl Drop for Library {
	fn drop(&mut self) {}
}

#[cfg(target_os = "windows")]
pub struct Window {
	hwnd: HWND,
}

#[cfg(target_os = "windows")]
impl Window {
	pub fn builder() -> WindowBuilder {
		WindowBuilder {
			title: None,
			width: 1280,
			height: 720,

			maximized: false,
			visible: true,
		}
	}
}

pub enum WindowError {
	ClassRegisterFailed,
}

pub struct WindowBuilder {
	title: Option<String>,
	width: u32,
	height: u32,

	maximized: bool,
	visible: bool,
}

impl WindowBuilder {
	pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
		self.title = Some(title.into());
		self
	}

	pub fn width(&mut self, width: u32) -> &mut Self {
		self.width = width;
		self
	}

	pub fn height(&mut self, height: u32) -> &mut Self {
		self.height = height;
		self
	}

	pub fn maximized(&mut self, maximized: bool) -> &mut Self {
		self.maximized = maximized;
		self
	}

	pub fn visible(&mut self, visible: bool) -> &mut Self {
		self.visible = visible;
		self
	}

	#[cfg(target_os = "windows")]
	pub fn spawn(&mut self) -> Result<Window, WindowError> {
		unsafe {
			let title = self.title.take().unwrap_or_else(|| String::from("title"));
			let window_class = winuser::WNDCLASSEXW {
				cbSize: std::mem::size_of::<winuser::WNDCLASSEXW>() as UINT,
				hInstance: GetModuleHandleA(std::ptr::null()),
				lpszClassName: OsStr::new(&title)
					.encode_wide()
					.collect::<Vec<u16>>()
					.as_ptr(),
				..Default::default()
			};

			let result = winuser::RegisterClassExW(&window_class);
			if result == 0 {
				return Err(WindowError::ClassRegisterFailed);
			}
		}
		Err(WindowError::ClassRegisterFailed)
	}
}
