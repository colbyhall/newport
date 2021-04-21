use crate::{ Id, Builder };

use std::{collections::hash_map::DefaultHasher, hash::Hasher};
use std::hash::Hash;

pub struct Button {
    pub id: Id,
    pub label: String,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        
        let mut hasher = DefaultHasher::new();
        Hash::hash(&label, &mut hasher);
        let id = hasher.finish();

        Self {
            id: Id(id),
            label: label,
        }
    }

    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let bounds = builder.layout.push_size(30.0);

        let is_over = builder.input().mouse_is_over(bounds);
        

        ButtonResponse::None
    }
}

pub enum ButtonResponse {
    None,
    Hovered,
    Clicked(u8),
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
            ButtonResponse::Clicked(_) => true,
            _ => false,
        }
    }
}