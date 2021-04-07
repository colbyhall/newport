use crate::{ Id, Input };

pub struct Context {
    hovered: Option<Id>,
    focused: Option<Id>,

    input:      Option<Input>,
    last_input: Option<Input>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            hovered: None,
            focused: None,

            input:      None,
            last_input: None,
        }
    }

    pub fn begin_frame(&mut self, input: Input) {
        self.last_input = self.input.take();
        self.input = Some(input);
    }

    pub fn end_frame(&mut self) {
        
    }
}