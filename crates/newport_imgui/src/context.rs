use crate::{ Id, Input };

pub struct Context {
    hovered: Option<Id>,
    focused: Option<Id>,

    input: Option<Input>,
}

