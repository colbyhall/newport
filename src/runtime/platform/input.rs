use winit::event::VirtualKeyCode;

/// Variant enum for `Input` used to distinguish between input types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputVariant {
	Unknown,
	Key {
		virtual_code: VirtualKeyCode,
		symbol: char,
	},
	MouseButton(u8),
	MouseAxis,
}

/// Static information about input sets
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Input {
	pub display_name: &'static str,
	pub variant: InputVariant,
}

impl Input {
	const fn key(display_name: &'static str, virtual_code: VirtualKeyCode, symbol: char) -> Self {
		Self {
			display_name,
			variant: InputVariant::Key {
				virtual_code,
				symbol,
			},
		}
	}

	const fn mouse_button(display_name: &'static str, index: u8) -> Self {
		Self {
			display_name,
			variant: InputVariant::MouseButton(index),
		}
	}

	const fn mouse_axis(display_name: &'static str) -> Self {
		Self {
			display_name,
			variant: InputVariant::MouseAxis,
		}
	}

	pub fn key_from_code(in_virtual_code: VirtualKeyCode) -> Option<Self> {
		for input in ALL_INPUTS.iter() {
			if let InputVariant::Key {
				virtual_code: code,
				symbol: _,
			} = input.variant
			{
				if in_virtual_code == code {
					return Some(*input);
				}
			}
		}

		None
	}

	pub fn as_key(self) -> (VirtualKeyCode, char) {
		match self.variant {
			InputVariant::Key {
				virtual_code: code,
				symbol,
			} => (code, symbol),
			_ => unreachable!(),
		}
	}

	pub fn as_mouse_button(self) -> u8 {
		match self.variant {
			InputVariant::MouseButton(code) => code,
			_ => unreachable!(),
		}
	}
}

pub const UNKNOWN: Input = Input {
	display_name: "Unknown",
	variant: InputVariant::Unknown,
};

pub const KEY_BACKSPACE: Input = Input::key("Backspace", VirtualKeyCode::Back, '\0');
pub const KEY_TAB: Input = Input::key("Tab", VirtualKeyCode::Tab, '\t');
pub const KEY_ENTER: Input = Input::key("Enter", VirtualKeyCode::Return, '\0');

pub const KEY_0: Input = Input::key("Zero Key", VirtualKeyCode::Key0, '0');
pub const KEY_1: Input = Input::key("One Key", VirtualKeyCode::Key1, '1');
pub const KEY_2: Input = Input::key("Two Key", VirtualKeyCode::Key2, '2');
pub const KEY_3: Input = Input::key("Three Key", VirtualKeyCode::Key3, '3');
pub const KEY_4: Input = Input::key("Four Key", VirtualKeyCode::Key4, '4');
pub const KEY_5: Input = Input::key("Five Key", VirtualKeyCode::Key5, '5');
pub const KEY_6: Input = Input::key("Six Key", VirtualKeyCode::Key6, '6');
pub const KEY_7: Input = Input::key("Seven Key", VirtualKeyCode::Key7, '7');
pub const KEY_8: Input = Input::key("Eight Key", VirtualKeyCode::Key8, '8');
pub const KEY_9: Input = Input::key("Nine Key", VirtualKeyCode::Key9, '9');

pub const KEY_A: Input = Input::key("A Key", VirtualKeyCode::A, 'A');
pub const KEY_B: Input = Input::key("B Key", VirtualKeyCode::B, 'B');
pub const KEY_C: Input = Input::key("C Key", VirtualKeyCode::C, 'C');
pub const KEY_D: Input = Input::key("D Key", VirtualKeyCode::D, 'D');
pub const KEY_E: Input = Input::key("E Key", VirtualKeyCode::E, 'E');
pub const KEY_F: Input = Input::key("F Key", VirtualKeyCode::F, 'F');
pub const KEY_G: Input = Input::key("G Key", VirtualKeyCode::G, 'G');
pub const KEY_H: Input = Input::key("H Key", VirtualKeyCode::H, 'H');
pub const KEY_I: Input = Input::key("I Key", VirtualKeyCode::I, 'I');
pub const KEY_J: Input = Input::key("J Key", VirtualKeyCode::J, 'J');
pub const KEY_K: Input = Input::key("K Key", VirtualKeyCode::K, 'K');
pub const KEY_L: Input = Input::key("L Key", VirtualKeyCode::L, 'L');
pub const KEY_M: Input = Input::key("M Key", VirtualKeyCode::M, 'M');
pub const KEY_N: Input = Input::key("N Key", VirtualKeyCode::N, 'N');
pub const KEY_O: Input = Input::key("O Key", VirtualKeyCode::O, 'O');
pub const KEY_P: Input = Input::key("P Key", VirtualKeyCode::P, 'P');
pub const KEY_Q: Input = Input::key("Q Key", VirtualKeyCode::Q, 'Q');
pub const KEY_R: Input = Input::key("R Key", VirtualKeyCode::R, 'R');
pub const KEY_S: Input = Input::key("S Key", VirtualKeyCode::S, 'S');
pub const KEY_T: Input = Input::key("T Key", VirtualKeyCode::T, 'T');
pub const KEY_U: Input = Input::key("U Key", VirtualKeyCode::U, 'U');
pub const KEY_V: Input = Input::key("V Key", VirtualKeyCode::V, 'V');
pub const KEY_W: Input = Input::key("W Key", VirtualKeyCode::W, 'W');
pub const KEY_X: Input = Input::key("X Key", VirtualKeyCode::X, 'X');
pub const KEY_Y: Input = Input::key("Y Key", VirtualKeyCode::Y, 'Y');
pub const KEY_Z: Input = Input::key("Z Key", VirtualKeyCode::Z, 'Z');

pub const KEY_ESCAPE: Input = Input::key("Escape Key", VirtualKeyCode::Escape, '\0');
pub const KEY_LSHIFT: Input = Input::key("Left Shift Key", VirtualKeyCode::LShift, '\0');
pub const KEY_LCTRL: Input = Input::key("Left Ctrl Key", VirtualKeyCode::LControl, '\0');
pub const KEY_LALT: Input = Input::key("Left Alt Key", VirtualKeyCode::LAlt, '\0');
pub const KEY_PAUSE: Input = Input::key("Pause Key", VirtualKeyCode::Pause, '\0');
pub const KEY_CAPITAL: Input = Input::key("Capital Key", VirtualKeyCode::Capital, '\0');
pub const KEY_SPACE: Input = Input::key("Space Key", VirtualKeyCode::Space, '\0');
pub const KEY_PRIOR: Input = Input::key("Prior Key", VirtualKeyCode::PrevTrack, '\0');
pub const KEY_NEXT: Input = Input::key("Next Key", VirtualKeyCode::NextTrack, '\0');
pub const KEY_END: Input = Input::key("End Key", VirtualKeyCode::End, '\0');
pub const KEY_HOME: Input = Input::key("Home Key", VirtualKeyCode::Home, '\0');
pub const KEY_LEFT: Input = Input::key("Left Key", VirtualKeyCode::Left, '\0');
pub const KEY_UP: Input = Input::key("Up Key", VirtualKeyCode::Up, '\0');
pub const KEY_RIGHT: Input = Input::key("Right Key", VirtualKeyCode::Right, '\0');
pub const KEY_DOWN: Input = Input::key("Down Key", VirtualKeyCode::Down, '\0');
pub const KEY_INSERT: Input = Input::key("Insert Key", VirtualKeyCode::Insert, '\0');
pub const KEY_DELETE: Input = Input::key("Delete Key", VirtualKeyCode::Delete, '\0');

// pub const KEY_NUMPAD0: Input = Input::key("Numpad 0 Key", 0x60, '0');
// pub const KEY_NUMPAD1: Input = Input::key("Numpad 1 Key", 0x61, '1');
// pub const KEY_NUMPAD2: Input = Input::key("Numpad 2 Key", 0x62, '2');
// pub const KEY_NUMPAD3: Input = Input::key("Numpad 3 Key", 0x63, '3');
// pub const KEY_NUMPAD4: Input = Input::key("Numpad 4 Key", 0x64, '4');
// pub const KEY_NUMPAD5: Input = Input::key("Numpad 5 Key", 0x65, '5');
// pub const KEY_NUMPAD6: Input = Input::key("Numpad 6 Key", 0x66, '6');
// pub const KEY_NUMPAD7: Input = Input::key("Numpad 7 Key", 0x67, '7');
// pub const KEY_NUMPAD8: Input = Input::key("Numpad 8 Key", 0x68, '8');
// pub const KEY_NUMPAD9: Input = Input::key("Numpad 9 Key", 0x69, '9');
// pub const KEY_MULTIPLY: Input = Input::key("* Key", 0x6A, '*');
// pub const KEY_ADD: Input = Input::key("+ Key", 0x6B, '+');
// pub const KEY_SEPARATOR: Input = Input::key("| Key", 0x6C, '|');
// pub const KEY_SUBTRACT: Input = Input::key("- Key", 0x6D, '-');
// pub const KEY_DECIMAL: Input = Input::key(". Key", 0x6E, '.');
// pub const KEY_DIVIDE: Input = Input::key("/ Key", 0x6F, '/');
pub const KEY_F1: Input = Input::key("F1 Key", VirtualKeyCode::F1, '\0');
pub const KEY_F2: Input = Input::key("F2 Key", VirtualKeyCode::F2, '\0');
pub const KEY_F3: Input = Input::key("F3 Key", VirtualKeyCode::F3, '\0');
pub const KEY_F4: Input = Input::key("F4 Key", VirtualKeyCode::F4, '\0');
pub const KEY_F5: Input = Input::key("F5 Key", VirtualKeyCode::F5, '\0');
pub const KEY_F6: Input = Input::key("F6 Key", VirtualKeyCode::F6, '\0');
pub const KEY_F7: Input = Input::key("F7 Key", VirtualKeyCode::F7, '\0');
pub const KEY_F8: Input = Input::key("F8 Key", VirtualKeyCode::F8, '\0');
pub const KEY_F9: Input = Input::key("F9 Key", VirtualKeyCode::F9, '\0');
pub const KEY_F10: Input = Input::key("F10 Key", VirtualKeyCode::F10, '\0');
pub const KEY_F11: Input = Input::key("F11 Key", VirtualKeyCode::F11, '\0');
pub const KEY_F12: Input = Input::key("F12 Key", VirtualKeyCode::F12, '\0');
pub const KEY_F13: Input = Input::key("F13 Key", VirtualKeyCode::F13, '\0');
pub const KEY_F14: Input = Input::key("F14 Key", VirtualKeyCode::F14, '\0');
pub const KEY_F15: Input = Input::key("F15 Key", VirtualKeyCode::F15, '\0');
pub const KEY_F16: Input = Input::key("F16 Key", VirtualKeyCode::F16, '\0');
pub const KEY_F17: Input = Input::key("F17 Key", VirtualKeyCode::F17, '\0');
pub const KEY_F18: Input = Input::key("F18 Key", VirtualKeyCode::F18, '\0');
pub const KEY_F19: Input = Input::key("F19 Key", VirtualKeyCode::F19, '\0');
pub const KEY_F20: Input = Input::key("F20 Key", VirtualKeyCode::F20, '\0');
pub const KEY_F21: Input = Input::key("F21 Key", VirtualKeyCode::F21, '\0');
pub const KEY_F22: Input = Input::key("F22 Key", VirtualKeyCode::F22, '\0');
pub const KEY_F23: Input = Input::key("F23 Key", VirtualKeyCode::F23, '\0');
pub const KEY_F24: Input = Input::key("F24 Key", VirtualKeyCode::F24, '\0');

// pub const KEY_TILDE: Input = Input::key("Tilde Key", 40, '~');

pub const MOUSE_BUTTON_LEFT: Input = Input::mouse_button("Left Mouse Button", 0);
pub const MOUSE_BUTTON_MIDDLE: Input = Input::mouse_button("Middle Mouse Button", 1);
pub const MOUSE_BUTTON_RIGHT: Input = Input::mouse_button("Right Mouse Button", 2);

pub const MOUSE_AXIS_X: Input = Input::mouse_axis("Mouse Axis X");
pub const MOUSE_AXIS_Y: Input = Input::mouse_axis("Mouse Axis Y");

pub const NUM_INPUTS: usize = 85;
pub const ALL_INPUTS: [Input; NUM_INPUTS] = [
	KEY_BACKSPACE,
	KEY_TAB,
	KEY_ENTER,
	KEY_0,
	KEY_1,
	KEY_2,
	KEY_3,
	KEY_4,
	KEY_5,
	KEY_6,
	KEY_7,
	KEY_8,
	KEY_9,
	KEY_A,
	KEY_B,
	KEY_C,
	KEY_D,
	KEY_E,
	KEY_F,
	KEY_G,
	KEY_H,
	KEY_I,
	KEY_J,
	KEY_K,
	KEY_L,
	KEY_M,
	KEY_N,
	KEY_O,
	KEY_P,
	KEY_Q,
	KEY_R,
	KEY_S,
	KEY_T,
	KEY_U,
	KEY_V,
	KEY_W,
	KEY_X,
	KEY_Y,
	KEY_Z,
	KEY_ESCAPE,
	KEY_LSHIFT,
	KEY_LCTRL,
	KEY_LALT,
	KEY_PAUSE,
	KEY_CAPITAL,
	KEY_SPACE,
	KEY_PRIOR,
	KEY_NEXT,
	KEY_END,
	KEY_HOME,
	KEY_LEFT,
	KEY_UP,
	KEY_RIGHT,
	KEY_DOWN,
	KEY_INSERT,
	KEY_DELETE,
	// KEY_NUMPAD0,
	// KEY_NUMPAD1,
	// KEY_NUMPAD2,
	// KEY_NUMPAD3,
	// KEY_NUMPAD4,
	// KEY_NUMPAD5,
	// KEY_NUMPAD6,
	// KEY_NUMPAD7,
	// KEY_NUMPAD8,
	// KEY_NUMPAD9,
	// KEY_MULTIPLY,
	// KEY_ADD,
	// KEY_SEPARATOR,
	// KEY_SUBTRACT,
	// KEY_DECIMAL,
	// KEY_DIVIDE,
	KEY_F1,
	KEY_F2,
	KEY_F3,
	KEY_F4,
	KEY_F5,
	KEY_F6,
	KEY_F7,
	KEY_F8,
	KEY_F9,
	KEY_F10,
	KEY_F11,
	KEY_F12,
	KEY_F13,
	KEY_F14,
	KEY_F15,
	KEY_F16,
	KEY_F17,
	KEY_F18,
	KEY_F19,
	KEY_F20,
	KEY_F21,
	KEY_F22,
	KEY_F23,
	KEY_F24,
	MOUSE_BUTTON_LEFT,
	MOUSE_BUTTON_MIDDLE,
	MOUSE_BUTTON_RIGHT,
	MOUSE_AXIS_X,
	MOUSE_AXIS_Y,
];
