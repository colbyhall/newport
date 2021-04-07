use crate::{ GUI, Response };

mod label;
pub use label::*;

pub trait Widget {
    fn gui(self, gui: &mut GUI) -> Response;
}