use crate::{
    Builder, 
    Id, 
    ToId, 

    math,
    asset,
    graphics,
};

use math::{ Rect, Vector2 };
use asset::AssetRef;
use graphics::Texture;


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

pub struct Button {
    id: Id,
    label: String,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            id:    Id::from(&label),
            label: label,
        }
    }

    pub fn id(mut self, id: impl ToId) -> Self {
        self.id = id.to_id();
        self
    }
}

impl Button {
    #[must_use = "If a response is not being used then use a label"]
    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let style = builder.style();

        let label_rect = style.string_rect(&self.label, style.label_size, None).size();
        let bounds = builder.content_bounds(label_rect);
        
        let response = button_control(self.id, bounds, builder);
        
        let is_focused = builder.is_focused(self.id);
        let is_hovered = builder.is_hovered(self.id);
        
        let (background_color, foreground_color) = {
            let background_color = if is_focused {
                style.focused_background
            } else if is_hovered {
                style.hovered_background
            } else {
                style.unhovered_background
            };

            let foreground_color = if is_focused {
                style.focused_foreground
            } else if is_hovered {
                style.hovered_foreground
            } else {
                style.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.rect(bounds).color(background_color);
        
        let at = Rect::from_pos_size(bounds.pos(), label_rect).top_left();
        builder.painter
            .text(self.label, at, &style.font, style.label_size, builder.input().dpi)
            .color(foreground_color)
            .scissor(bounds);

        response
    }
}

pub struct ImageButton {
    id:    Id,
    image: AssetRef<Texture>,
    size:  Option<Vector2>,
}

impl ImageButton {
    pub fn new(image: &AssetRef<Texture>) -> Self {
        Self {
            id:    Id::from(image.path()),
            image: image.clone(),
            size:  None,
        }
    }

    pub fn id(mut self, id: impl ToId) -> Self {
        self.id = id.to_id();
        self
    }

    pub fn size(mut self, size: impl Into<Vector2>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl ImageButton {
    #[must_use = "If a response is not being used then use an image"]
    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let style = builder.style();

        let size = match self.size {
            Some(size) => size,
            None => {
                let image = self.image.read();
                Vector2::new(image.gpu().width() as f32, image.gpu().height() as f32)
            }
        };
        let bounds = builder.content_bounds(size);
        
        let response = button_control(self.id, bounds, builder);
        
        let is_focused = builder.is_focused(self.id);
        let is_hovered = builder.is_hovered(self.id);
        
        let (background_color, foreground_color) = {
            let background_color = if is_focused {
                style.focused_background
            } else if is_hovered {
                style.hovered_background
            } else {
                style.unhovered_background
            };

            let foreground_color = if is_focused {
                style.focused_foreground
            } else if is_hovered {
                style.hovered_foreground
            } else {
                style.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.rect(bounds).color(background_color);
        
        let gpu_texture = {
            let texture = self.image.read();
            texture.gpu().clone()
        };

        let bounds = Rect::from_pos_size(bounds.pos(), size);
        builder.painter.rect(bounds).color(foreground_color).texture(&gpu_texture);

        response
    }
}