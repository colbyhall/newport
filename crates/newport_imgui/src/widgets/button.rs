use crate::{ Id, Builder, ButtonResponse, button_control };
use crate::math::Rect;

pub struct Button {
    pub id: Id,
    pub label: String,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            id:    Id::from(&label),
            label: label,
        }
    }

    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        
        let style = builder.style().clone();
        let spacing = builder.spacing().clone();
        
        let label_rect = style.string_rect(&self.label, style.label_size, None);
        let size = label_rect.size() + spacing.padding.min + spacing.padding.max;
        
        let layout_rect = builder.layout.push_size(size + spacing.margin.min + spacing.margin.max);
        let bounds = Rect::from_pos_size(layout_rect.pos(), size);
        
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
        
        let at = Rect::from_pos_size(bounds.pos(), label_rect.size()).top_left();
        builder.painter
            .text(self.label, at, &style.font, style.label_size, builder.input().dpi)
            .color(foreground_color)
            .scissor(bounds);

        response
    }
}