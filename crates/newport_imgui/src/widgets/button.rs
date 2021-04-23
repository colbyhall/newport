use crate::{Builder, ButtonResponse, Id, Organization, ToId, button_control};
use crate::math::Rect;

pub struct Button {
    id: Id,
    label: String,
    
    organization: Option<Organization>
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            id:    Id::from(&label),
            label: label,

            organization: None,
        }
    }

    pub fn id(mut self, id: impl ToId) -> Self {
        self.id = id.to_id();
        self
    }

    pub fn organization(mut self, organization: Organization) -> Self {
        self.organization = Some(organization);
        self
    }
}

impl Button {
    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let style = builder.style();
        let organization = self.organization.unwrap_or(builder.organization());
        
        let label_rect = style.string_rect(&self.label, style.label_size, None);
        let size = organization.content_size(label_rect.size(), builder.layout.space_left());
        
        let layout_rect = builder.layout.push_size(organization.spacing_size(size));
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