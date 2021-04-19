use crate::{ Id, Builder };

pub struct Button {
    id: Id,
    label: String,
}

impl Button {
    pub fn build(&self, builder: &mut Builder) -> ButtonResponse {
        
    }
}

pub enum ButtonResponse {
    None,
    Hovered,
    Clicked,
}

impl ButtonResponse {
    pub fn hovered(&self) -> bool {
        match self {
            ButtonResponse::Hovered => true,
            _ => false,
        }
    }

    pub fn clicked(&self) -> bool {
        match self {
            ButtonResponse::Hovered => true,
            _ => false,
        }
    }
}