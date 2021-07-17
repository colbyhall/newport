#[cfg(target_os = "windows")]
pub mod win32;

pub mod dialog;
pub mod input;
pub mod library;
pub mod time;
pub mod window;

pub fn caret_blink_time() -> f32 {
	(unsafe { win32::GetCaretBlinkTime() } as f32) / 1000.0
}
