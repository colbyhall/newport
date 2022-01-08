/// Variant enum for `Input` used to distinguish between input types
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputVariant {
	Unknown,
	Key,
	MouseButton,
	MouseAxis,
}

/// Static information about input sets
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Input {
	pub display_name: &'static str,
	pub variant: InputVariant,
}

impl Input {
	const fn key(display_name: &'static str) -> Self {
		Self {
			display_name,
			variant: InputVariant::Key,
		}
	}

	const fn mouse_button(display_name: &'static str) -> Self {
		Self {
			display_name,
			variant: InputVariant::MouseButton,
		}
	}

	const fn mouse_axis(display_name: &'static str) -> Self {
		Self {
			display_name,
			variant: InputVariant::MouseAxis,
		}
	}
}

pub const UNKNOWN: Input = Input {
	display_name: "Unknown",
	variant: InputVariant::Unknown,
};

pub const KEY_BACKSPACE: Input = Input::key("Backspace");
pub const KEY_TAB: Input = Input::key("Tab");
pub const KEY_ENTER: Input = Input::key("Enter");

pub const KEY_0: Input = Input::key("Zero");
pub const KEY_1: Input = Input::key("One");
pub const KEY_2: Input = Input::key("Two");
pub const KEY_3: Input = Input::key("Three");
pub const KEY_4: Input = Input::key("Four");
pub const KEY_5: Input = Input::key("Five");
pub const KEY_6: Input = Input::key("Six");
pub const KEY_7: Input = Input::key("Seven");
pub const KEY_8: Input = Input::key("Eight");
pub const KEY_9: Input = Input::key("Nine");

pub const KEY_A: Input = Input::key("A");
pub const KEY_B: Input = Input::key("B");
pub const KEY_C: Input = Input::key("C");
pub const KEY_D: Input = Input::key("D");
pub const KEY_E: Input = Input::key("E");
pub const KEY_F: Input = Input::key("F");
pub const KEY_G: Input = Input::key("G");
pub const KEY_H: Input = Input::key("H");
pub const KEY_I: Input = Input::key("I");
pub const KEY_J: Input = Input::key("J");
pub const KEY_K: Input = Input::key("K");
pub const KEY_L: Input = Input::key("L");
pub const KEY_M: Input = Input::key("M");
pub const KEY_N: Input = Input::key("N");
pub const KEY_O: Input = Input::key("O");
pub const KEY_P: Input = Input::key("P");
pub const KEY_Q: Input = Input::key("Q");
pub const KEY_R: Input = Input::key("R");
pub const KEY_S: Input = Input::key("S");
pub const KEY_T: Input = Input::key("T");
pub const KEY_U: Input = Input::key("U");
pub const KEY_V: Input = Input::key("V");
pub const KEY_W: Input = Input::key("W");
pub const KEY_X: Input = Input::key("X");
pub const KEY_Y: Input = Input::key("Y");
pub const KEY_Z: Input = Input::key("Z");

pub const KEY_ESCAPE: Input = Input::key("Escape Key");
pub const KEY_LSHIFT: Input = Input::key("Left Shift");
pub const KEY_LCTRL: Input = Input::key("Left Ctrl");
pub const KEY_LALT: Input = Input::key("Left Alt");
pub const KEY_PAUSE: Input = Input::key("Pause");
pub const KEY_CAPS_LOCK: Input = Input::key("Caps Lock");
pub const KEY_SPACE: Input = Input::key("Space");
pub const KEY_PRIOR: Input = Input::key("Prior");
pub const KEY_NEXT: Input = Input::key("Next");
pub const KEY_END: Input = Input::key("End");
pub const KEY_HOME: Input = Input::key("Home");
pub const KEY_LEFT: Input = Input::key("Left");
pub const KEY_UP: Input = Input::key("Up");
pub const KEY_RIGHT: Input = Input::key("Right");
pub const KEY_DOWN: Input = Input::key("Down");
pub const KEY_INSERT: Input = Input::key("Insert");
pub const KEY_DELETE: Input = Input::key("Delete");

pub const KEY_NUMPAD0: Input = Input::key("Numpad 0");
pub const KEY_NUMPAD1: Input = Input::key("Numpad 1");
pub const KEY_NUMPAD2: Input = Input::key("Numpad 2");
pub const KEY_NUMPAD3: Input = Input::key("Numpad 3");
pub const KEY_NUMPAD4: Input = Input::key("Numpad 4");
pub const KEY_NUMPAD5: Input = Input::key("Numpad 5");
pub const KEY_NUMPAD6: Input = Input::key("Numpad 6");
pub const KEY_NUMPAD7: Input = Input::key("Numpad 7");
pub const KEY_NUMPAD8: Input = Input::key("Numpad 8");
pub const KEY_NUMPAD9: Input = Input::key("Numpad 9");
pub const KEY_MULTIPLY: Input = Input::key("Multiply");
pub const KEY_ADD: Input = Input::key("Add");
pub const KEY_SEPARATOR: Input = Input::key("Seperator");
pub const KEY_SUBTRACT: Input = Input::key("Subtract");
pub const KEY_PERIOD: Input = Input::key("Period");
pub const KEY_DIVIDE: Input = Input::key("/ Key");
pub const KEY_F1: Input = Input::key("F1");
pub const KEY_F2: Input = Input::key("F2");
pub const KEY_F3: Input = Input::key("F3");
pub const KEY_F4: Input = Input::key("F4");
pub const KEY_F5: Input = Input::key("F5");
pub const KEY_F6: Input = Input::key("F6");
pub const KEY_F7: Input = Input::key("F7");
pub const KEY_F8: Input = Input::key("F8");
pub const KEY_F9: Input = Input::key("F9");
pub const KEY_F10: Input = Input::key("F10");
pub const KEY_F11: Input = Input::key("F11");
pub const KEY_F12: Input = Input::key("F12");
pub const KEY_F13: Input = Input::key("F13");
pub const KEY_F14: Input = Input::key("F14");
pub const KEY_F15: Input = Input::key("F15");
pub const KEY_F16: Input = Input::key("F16");
pub const KEY_F17: Input = Input::key("F17");
pub const KEY_F18: Input = Input::key("F18");
pub const KEY_F19: Input = Input::key("F19");
pub const KEY_F20: Input = Input::key("F20");
pub const KEY_F21: Input = Input::key("F21");
pub const KEY_F22: Input = Input::key("F22");
pub const KEY_F23: Input = Input::key("F23");
pub const KEY_F24: Input = Input::key("F24");

// pub const KEY_TILDE: Input = Input::key("Tilde Key", 40, '~');

pub const MOUSE_BUTTON_LEFT: Input = Input::mouse_button("Left Mouse Button");
pub const MOUSE_BUTTON_MIDDLE: Input = Input::mouse_button("Middle Mouse Button");
pub const MOUSE_BUTTON_RIGHT: Input = Input::mouse_button("Right Mouse Button");

pub const MOUSE_AXIS_X: Input = Input::mouse_axis("Mouse Axis X");
pub const MOUSE_AXIS_Y: Input = Input::mouse_axis("Mouse Axis Y");
