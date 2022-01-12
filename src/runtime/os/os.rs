pub mod input;
pub use input::*;

pub mod time;

#[cfg(target_os = "windows")]
pub mod windows {
	pub use winapi::*;
}

pub use winit::{
	self,
	event::{
		Event,
		VirtualKeyCode,
		WindowEvent,
	},
	event_loop::{
		ControlFlow,
		EventLoop,
	},
	window::{
		Window,
		WindowBuilder,
	},
};

pub use raw_window_handle::{
	HasRawWindowHandle,
	RawWindowHandle,
};

#[cfg(target_os = "windows")]
use std::{
	ffi::OsStr,
	iter::Iterator,
	ops::Deref,
	os::windows::ffi::OsStrExt,
};

#[cfg(target_os = "windows")]
use windows::{
	shared::minwindef::{
		FARPROC,
		HMODULE,
	},
	um::libloaderapi::{
		FreeLibrary,
		GetProcAddress,
		LoadLibraryW,
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

pub fn virtual_keycode_to_input(vk: VirtualKeyCode) -> Input {
	const VK_MAP: &[Input] = &[
		KEY_1,         // Key1,
		KEY_2,         // Key2,
		KEY_3,         // Key3,
		KEY_4,         // Key4,
		KEY_5,         // Key5,
		KEY_6,         // Key6,
		KEY_7,         // Key7,
		KEY_8,         // Key8,
		KEY_9,         // Key9,
		KEY_0,         // Key0,
		KEY_A,         // A,
		KEY_B,         // B,
		KEY_C,         // C,
		KEY_D,         // D,
		KEY_E,         // E,
		KEY_F,         // F,
		KEY_G,         // G,
		KEY_H,         // H,
		KEY_I,         // I,
		KEY_J,         // J,
		KEY_K,         // K,
		KEY_L,         // L,
		KEY_M,         // M,
		KEY_N,         // N,
		KEY_O,         // O,
		KEY_P,         // P,
		KEY_Q,         // Q,
		KEY_R,         // R,
		KEY_S,         // S,
		KEY_T,         // T,
		KEY_U,         // U,
		KEY_V,         // V,
		KEY_W,         // W,
		KEY_X,         // X,
		KEY_Y,         // Y,
		KEY_Z,         // Z,
		KEY_ESCAPE,    // Escape,
		KEY_F1,        // F1,
		KEY_F2,        // F2,
		KEY_F3,        // F3,
		KEY_F4,        // F4,
		KEY_F5,        // F5,
		KEY_F6,        // F6,
		KEY_F7,        // F7,
		KEY_F8,        // F8,
		KEY_F9,        // F9,
		KEY_F10,       // F10,
		KEY_F11,       // F11,
		KEY_F12,       // F12,
		KEY_F13,       // F13,
		KEY_F14,       // F14,
		KEY_F15,       // F15,
		KEY_F16,       // F16,
		KEY_F17,       // F17,
		KEY_F18,       // F18,
		KEY_F19,       // F19,
		KEY_F20,       // F20,
		KEY_F21,       // F21,
		KEY_F22,       // F22,
		KEY_F23,       // F23,
		KEY_F24,       // F24,
		UNKNOWN,       // Snapshot,
		UNKNOWN,       // Scroll,
		UNKNOWN,       // Pause,
		KEY_INSERT,    // Insert,
		KEY_HOME,      // Home,
		KEY_DELETE,    // Delete,
		KEY_END,       // End,
		UNKNOWN,       // PageDown,
		UNKNOWN,       // PageUp,
		KEY_LEFT,      // Left,
		KEY_UP,        // Up,
		KEY_RIGHT,     // Right,
		KEY_DOWN,      // Down,
		KEY_BACKSPACE, // Back,
		KEY_ENTER,     // Return,
		KEY_SPACE,     // Space,
		UNKNOWN,       // Compose,
		UNKNOWN,       // Caret,
		UNKNOWN,       // Numlock,
		KEY_NUMPAD0,   // Numpad0,
		KEY_NUMPAD1,   // Numpad1,
		KEY_NUMPAD2,   // Numpad2,
		KEY_NUMPAD3,   // Numpad3,
		KEY_NUMPAD4,   // Numpad4,
		KEY_NUMPAD5,   // Numpad5,
		KEY_NUMPAD6,   // Numpad6,
		KEY_NUMPAD7,   // Numpad7,
		KEY_NUMPAD8,   // Numpad8,
		KEY_NUMPAD9,   // Numpad9,
		UNKNOWN,       // NumpadAdd,
		UNKNOWN,       // NumpadDivide,
		UNKNOWN,       // NumpadDecimal,
		UNKNOWN,       // NumpadComma,
		UNKNOWN,       // NumpadEnter,
		UNKNOWN,       // NumpadEquals,
		UNKNOWN,       // NumpadMultiply,
		UNKNOWN,       // NumpadSubtract,
		UNKNOWN,       // AbntC1,
		UNKNOWN,       // AbntC2,
		UNKNOWN,       // Apostrophe,
		UNKNOWN,       // Apps,
		UNKNOWN,       // Asterisk,
		UNKNOWN,       // At,
		UNKNOWN,       // Ax,
		UNKNOWN,       // Backslash,
		UNKNOWN,       // Calculator,
		UNKNOWN,       // Capital,
		UNKNOWN,       // Colon,
		UNKNOWN,       // Comma,
		UNKNOWN,       // Convert,
		UNKNOWN,       // Equals,
		UNKNOWN,       // Grave,
		UNKNOWN,       // Kana,
		UNKNOWN,       // Kanji,
		UNKNOWN,       // LAlt,
		UNKNOWN,       // LBracket,
		KEY_LCTRL,     // LControl,
		UNKNOWN,       // LShift,
		UNKNOWN,       // LWin,
		UNKNOWN,       // Mail,
		UNKNOWN,       // MediaSelect,
		UNKNOWN,       // MediaStop,
		UNKNOWN,       // Minus,
		UNKNOWN,       // Mute,
		UNKNOWN,       // MyComputer,
		UNKNOWN,       // // also called "Next"
		UNKNOWN,       // NavigateForward,
		UNKNOWN,       // // also called "Prior"
		UNKNOWN,       // NavigateBackward,
		UNKNOWN,       // NextTrack,
		UNKNOWN,       // NoConvert,
		UNKNOWN,       // OEM102,
		UNKNOWN,       // Period,
		UNKNOWN,       // PlayPause,
		UNKNOWN,       // Plus,
		UNKNOWN,       // Power,
		UNKNOWN,       // PrevTrack,
		UNKNOWN,       // RAlt,
		UNKNOWN,       // RBracket,
		UNKNOWN,       // RControl,
		UNKNOWN,       // RShift,
		UNKNOWN,       // RWin,
		UNKNOWN,       // Semicolon,
		UNKNOWN,       // Slash,
		UNKNOWN,       // Sleep,
		UNKNOWN,       // Stop,
		UNKNOWN,       // Sysrq,
		KEY_TAB,       // Tab,
		UNKNOWN,       // Underline,
		UNKNOWN,       // Unlabeled,
		UNKNOWN,       // VolumeDown,
		UNKNOWN,       // VolumeUp,
		UNKNOWN,       // Wake,
		UNKNOWN,       // WebBack,
		UNKNOWN,       // WebFavorites,
		UNKNOWN,       // WebForward,
		UNKNOWN,       // WebHome,
		UNKNOWN,       // WebRefresh,
		UNKNOWN,       // WebSearch,
		UNKNOWN,       // WebStop,
		UNKNOWN,       // Yen,
		UNKNOWN,       // Copy,
		UNKNOWN,       // Paste,
		UNKNOWN,       // Cut,
	];

	VK_MAP[vk as u32 as usize]
}
