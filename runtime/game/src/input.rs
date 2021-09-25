use math::Vector2;
use platform::input::Input;

pub struct InputState {
	pub mouse_location: Option<Vector2>,
	pub last_mouse_location: Option<Vector2>,

	pub key_down: [bool; 256],
	pub last_key_down: [bool; 256],

	pub mouse_button_down: [bool; 3],
	pub last_mouse_button_down: [bool; 3],

	pub mouse_delta: Vector2,

	pub mouse_locked: bool,
}

impl InputState {
	pub fn is_key_down(&self, key: Input) -> bool {
		self.key_down[key.as_key().0 as usize]
	}

	pub fn was_key_pressed(&self, key: Input) -> bool {
		!self.last_key_down[key.as_key().0 as usize] && self.key_down[key.as_key().0 as usize]
	}
}

impl Default for InputState {
	fn default() -> Self {
		Self {
			mouse_location: None,
			last_mouse_location: None,

			key_down: [false; 256],
			last_key_down: [false; 256],

			mouse_button_down: [false; 3],
			last_mouse_button_down: [false; 3],

			mouse_delta: Vector2::ZERO,

			mouse_locked: false,
		}
	}
}
