#[cfg(windows)]
pub mod win32;

pub mod dialog;
pub mod input;
pub mod library;
pub mod time;
pub mod window;

#[cfg(windows)]
pub fn caret_blink_time() -> f32 {
    (unsafe { win32::GetCaretBlinkTime() } as f32) / 1000.0
}

#[cfg(target_os = "linux")]
pub fn caret_blink_time() -> f32 {
    // levy: Im not sure if x11 has a caret blink time
    // TODO: figure this out
    5.
}
