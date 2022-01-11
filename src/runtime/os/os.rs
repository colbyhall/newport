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
	iter::Iterator,
	num::Wrapping,
	ops::{
		Deref,
		DerefMut,
	},
	os::windows::ffi::OsStrExt,
};

#[cfg(target_os = "windows")]
use windows::{
	shared::{
		minwindef::{
			FARPROC,
			HMODULE,
			LPARAM,
			LRESULT,
			UINT,
			WPARAM,
		},
		windef::{
			HWND,
			RECT,
		},
	},
	um::{
		errhandlingapi::GetLastError,
		libloaderapi::{
			FreeLibrary,
			GetModuleHandleA,
			GetProcAddress,
			LoadLibraryW,
		},
		winuser,
	},
};

#[cfg(target_os = "windows")]
pub struct Symbol<'a, T> {
	ptr: FARPROC,
	phantom: std::marker::PhantomData<&'a T>,
}

unsafe impl<'a, T: Send> Send for Symbol<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Symbol<'a, T> {}

impl<'a, T> Clone for Symbol<'a, T> {
	fn clone(&self) -> Self {
		Symbol { ..*self }
	}
}

impl<'a, T> Deref for Symbol<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe { &*(&self.ptr as *const *mut _ as *const T) }
	}
}

#[derive(Debug)]
pub enum LibraryError {
	NotFound,
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
				Err(LibraryError::NotFound)
			} else {
				Ok(Self { hmodule })
			}
		}
	}

	pub fn get<T>(&self, symbol: &str) -> Result<Symbol<'_, T>, LibraryError> {
		assert!(symbol.is_ascii());
		let ptr = unsafe { GetProcAddress(self.hmodule, symbol.as_ptr() as *const i8) };
		if ptr.is_null() {
			Err(LibraryError::NotFound)
		} else {
			Ok(Symbol {
				ptr,
				phantom: std::marker::PhantomData,
			})
		}
	}
}

unsafe impl Send for Library {}
unsafe impl Sync for Library {}

impl Drop for Library {
	fn drop(&mut self) {
		let result = unsafe { FreeLibrary(self.hmodule) };
		assert!(result > 0);
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg(target_os = "windows")]
pub struct WindowId(HWND);

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

	pub fn set_visible(&mut self, visible: bool) {
		let cmd = if visible {
			winuser::SW_SHOW
		} else {
			winuser::SW_HIDE
		};
		unsafe { winuser::ShowWindow(self.hwnd, cmd) };
	}

	pub fn set_maximized(&mut self, maximized: bool) {
		let cmd = if maximized {
			winuser::SW_MAXIMIZE
		} else {
			winuser::SW_NORMAL
		};
		unsafe { winuser::ShowWindow(self.hwnd, cmd) };
	}

	pub fn id(&self) -> WindowId {
		WindowId(self.hwnd)
	}
}

#[derive(Debug)]
pub enum WindowError {
	ClassRegisterFailed,
	CreationFailed,
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
	pub fn spawn(&mut self, event_handler: &mut WindowEventHandler) -> Result<Window, WindowError> {
		unsafe {
			let title = self.title.take().unwrap_or_else(|| String::from("Window"));

			let mut class_name: Vec<u16> = OsStr::new("class_name\0").encode_wide().collect();
			class_name.push(0);

			let window_class = winuser::WNDCLASSEXW {
				cbSize: std::mem::size_of::<winuser::WNDCLASSEXW>() as UINT,
				hInstance: GetModuleHandleA(std::ptr::null()),
				lpfnWndProc: Some(window_proc),
				lpszClassName: class_name.as_ptr(),
				..Default::default()
			};

			let result = winuser::RegisterClassExW(&window_class);
			if result == 0 {
				return Err(WindowError::ClassRegisterFailed);
			}

			let width = self.width;
			let height = self.height;

			let mut adjusted_rect = RECT {
				left: 0,
				top: 0,
				right: width as _,
				bottom: height as _,
			};

			let style = winuser::WS_OVERLAPPEDWINDOW;
			let result = winuser::AdjustWindowRect(&mut adjusted_rect, style, 0);
			if result == 0 {
				todo!("Error handling");
			}

			let width = (Wrapping(adjusted_rect.right) - Wrapping(adjusted_rect.left)).0;
			let height = (Wrapping(adjusted_rect.bottom) - Wrapping(adjusted_rect.top)).0;

			let monitor_width = winuser::GetSystemMetrics(winuser::SM_CXSCREEN);
			let monitor_height = winuser::GetSystemMetrics(winuser::SM_CYSCREEN);

			let x = monitor_width / 2 - width / 2;
			let y = monitor_height / 2 - height / 2;

			let mut window_name: Vec<u16> = OsStr::new(&title).encode_wide().collect();
			window_name.push(0);
			let hwnd = winuser::CreateWindowExW(
				0,
				window_class.lpszClassName,
				window_name.as_ptr(),
				style,
				x,
				y,
				width,
				height,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				window_class.hInstance,
				std::ptr::null_mut(),
			);

			if hwnd.is_null() {
				println!("{:#x?}", GetLastError());
				return Err(WindowError::CreationFailed);
			}

			// winuser::SetWindowLongPtrW(
			// 	hwnd,
			// 	winuser::GWLP_USERDATA,
			// 	event_handler.0.deref_mut() as *mut Option<WindowEvent> as isize,
			// );

			if self.visible {
				winuser::ShowWindow(hwnd, winuser::SW_SHOWNORMAL);
			}

			if self.maximized {
				winuser::ShowWindow(hwnd, winuser::SW_MAXIMIZE);
			}

			Ok(Window { hwnd })
		}
	}
}

unsafe extern "system" fn window_proc(
	hwnd: HWND,
	msg: UINT,
	wparam: WPARAM,
	lparam: LPARAM,
) -> LRESULT {
	let event_handler = &mut *(winuser::GetWindowLongPtrW(hwnd, winuser::GWLP_USERDATA) as usize
		as *const u8 as *mut dyn FnMut(WindowEvent));

	let window = WindowId(hwnd);

	match msg {
		winuser::WM_DESTROY => {
			*event_handler = Some(WindowEvent {
				window,
				variant: WindowEventVariant::Destroyed,
			})
		}
		winuser::WM_CLOSE => {
			*event_handler = Some(WindowEvent {
				window,
				variant: WindowEventVariant::ExitRequested,
			})
		}
		_ => {}
	}
	winuser::DefWindowProcW(hwnd, msg, wparam, lparam)
}

#[derive(Clone, Copy)]
pub enum WindowEventVariant {
	ExitRequested,
	Destroyed,
	FocusGained,
	FocusLost,
	Key { key: Input, pressed: bool },
	Resized(u32, u32),
	Char(char),
	MouseWheel(f32, f32),
	MouseButton { mouse_button: Input, pressed: bool },
	MouseMove(u32, u32),
	MouseLeave,
	MouseEnter,
}

#[derive(Clone, Copy)]
pub struct WindowEvent {
	pub window: WindowId,
	pub variant: WindowEventVariant,
}

#[derive(Default)]
pub struct WindowEventHandler(Box<usize>);

impl WindowEventHandler {
	pub fn new() -> Self {
		Self(Box::new(0))
	}

	pub fn poll(&mut self, mut event_handler: impl FnMut(WindowEvent)) {
		unsafe {
			let foo = &mut event_handler as *mut dyn FnMut(WindowEvent) as *mut u8 as usize;

			let mut msg = std::mem::zeroed();
			while winuser::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, winuser::PM_REMOVE)
				> 0
			{
				winuser::TranslateMessage(&msg);
				winuser::DispatchMessageW(&msg);
			}
		}
	}
}

#[test]
fn foo() {
	let mut event_handler = WindowEventHandler::new();
	let _window = Window::builder()
		.visible(true)
		.title("Hello World💖")
		.spawn(&mut event_handler)
		.unwrap();

	let mut running = true;
	while running {
		event_handler.poll(|event| match event.variant {
			WindowEventVariant::Destroyed | WindowEventVariant::ExitRequested => running = false,
			_ => println!("foo"),
		})
	}
}
