use newport_math::Vector2;

#[derive(Clone)]
pub struct Input {
    pub mouse_pos:     Vector2,
    pub mouse_buttons: [bool; 3],

    pub text_input: [char; 16],
    pub down_keys:  [bool; 255],
}

impl Default for Input {
    fn default() -> Self {
        Self {
            mouse_pos:      Vector2::default(),
            mouse_buttons:  [false; 3],

            text_input: ['\0'; 16],
            down_keys:  [false; 255]
        }
    }
}