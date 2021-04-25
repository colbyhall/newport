use crate::{
    Builder,
    ButtonResponse,
    Id,
    ToId,
    button_control,
};

use crate::math::{ Rect, Vector2 };

pub struct Checkbox<'a> {
    id: Id,
    is_checked: &'a mut bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(id: impl ToId, is_checked: &'a mut bool) -> Self {
        Self{
            id: id.to_id(),
            is_checked: is_checked,
        }
    }
}

impl<'a> Checkbox<'a> {
    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let style = builder.style();

        let size = style.label_height();

        let checkbox_size = Vector2::new(size, size);
        let check_size = (size / 3.0, size / 3.0).into();

        let bounds = Rect::from_pos_size(builder.content_bounds(checkbox_size).pos(), checkbox_size);

        let response = button_control(self.id, bounds, builder);

        if response.clicked() {
            *self.is_checked = !*self.is_checked;
        }
        
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

        if *self.is_checked {
            let check_bounds = Rect::from_pos_size(bounds.pos(), check_size);
            builder.painter.rect(check_bounds).color(foreground_color);
        }

        response
    }
}