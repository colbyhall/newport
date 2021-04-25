use crate::{ Builder, Id };
use crate::math::Rect;

#[derive(Copy, Clone)]
pub enum ButtonResponse {
    None,
    Hovered,
    Clicked(u8),
}

impl ButtonResponse {
    pub fn hovered(self) -> bool {
        match self {
            ButtonResponse::Hovered => true,
            _ => false,
        }
    }

    pub fn clicked(self) -> bool {
        match self {
            ButtonResponse::Clicked(_) => true,
            _ => false,
        }
    }
}

pub fn button_control(id: Id, bounds: Rect, builder: &mut Builder) -> ButtonResponse {
    let mut response = ButtonResponse::None;
    let is_over = builder.input().mouse_is_over(bounds);
    if is_over {
        if !builder.is_hovered(id) {
            response = ButtonResponse::Hovered;
        }
        builder.hover(id);

        if builder.input().was_primary_clicked() {
            builder.focus(id);
        }
    } else {
        builder.unhover(id);
    }

    if builder.input().was_primary_released() {
        if builder.unfocus(id) && is_over {
            response = ButtonResponse::Clicked(0);
        }
    }

    response
}