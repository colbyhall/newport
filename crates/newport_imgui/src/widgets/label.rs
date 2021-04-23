use crate::{ Builder };

use crate::math::{ Color, Vector2, Rect };

pub struct Label {
    label: String,

    color:  Option<Color>,
    _size:   Option<u32>,
    _bounds: Option<Rect>,
}

impl Label {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            label: label,
            
            color:  None,
            _size:   None,
            _bounds: None,
        }
    }
}

impl Label {
    pub fn build(self, builder: &mut Builder) {
        let style = builder.style();
        let organization = builder.organization();
        
        let label_rect = style.string_rect(&self.label, style.label_size, None);
        let size = organization.content_size(label_rect.size(), builder.layout.space_left());
        
        let layout_rect = builder.layout.push_size(organization.spacing_size(size));
        let bounds = Rect::from_pos_size(layout_rect.pos(), size);

        let at = Rect::from_pos_size(bounds.pos(), label_rect.size()).top_left();

        builder.painter
            .text(self.label, at, &style.font, style.label_size, builder.input().dpi)
            .color(self.color.unwrap_or(style.inactive_foreground))
            .scissor(bounds);
    }
}