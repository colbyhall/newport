pub mod input;
pub mod time;

use winapi::um::winuser::GetCaretBlinkTime;

pub fn caret_blink_time() -> f32 {
	(unsafe { GetCaretBlinkTime() } as f32) / 1000.0
}

pub use raw_window_handle;
pub use winapi;
pub use winit;
