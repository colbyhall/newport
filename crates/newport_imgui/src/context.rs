use crate::{ Id, Control };

pub struct Context {
    frame: u64,
}

impl Context {
    pub fn new() -> Self {
        Self {
            frame:  0,
        }
    }

    pub fn begin_frame(&mut self) {
        
    }

    pub fn end_frame(&mut self) {

    }
}