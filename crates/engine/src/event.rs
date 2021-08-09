use platform::input::Input;

#[derive(Clone)]
pub enum Event {
	FocusGained,
	FocusLost,
	Key { key: Input, pressed: bool },
	Resized(u32, u32),
	Char(char),
	MouseWheel(f32, f32),
	MouseButton { mouse_button: Input, pressed: bool },
	MouseMove(f32, f32),
	MouseLeave,
	MouseEnter,
}
