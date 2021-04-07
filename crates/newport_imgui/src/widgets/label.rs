use newport_math::Color;

use crate::{ Widget, GUI, Response };

use std::convert::Into;

pub struct Label {
    text: String,

    background_color: Option<Color>,
    text_color:       Option<Color>,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),

            background_color: None,
            text_color:       None,
        }
    }
}

impl Widget for Label {
    fn gui(self, gui: &mut GUI) -> Response {

        Response::NONE
    }
}