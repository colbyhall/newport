#[cfg(target_os = "windows")]
pub mod win32;

pub mod window;
pub mod input;
pub mod time;
pub mod library;

pub fn caret_blink_time() -> f32 {
    (unsafe{ win32::GetCaretBlinkTime() } as f32) / 1000.0
}