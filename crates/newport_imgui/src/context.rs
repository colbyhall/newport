use crate::{ Id, Input };

pub struct Context {
    hovered: Option<Id>,
    focused: Option<Id>,

    input: Option<Input>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            hovered: None,
            focused: None,
        }
    }

    pub fn begin_frame(&mut self, input: Input) -> {
        
    }

    pub fn end_frame(&mut self) {

    }
}