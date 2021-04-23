use newport_os::input::KEY_SPACE;

use crate::{ Builder };

use crate::math::{ Color, Vector2, Rect };

pub struct Label {
    label: String,

    color:  Option<Color>,
    size:   Option<u32>,
    bounds: Option<Rect>,
}

impl Label {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            label: label,
            
            color:  None,
            size:   None,
            bounds: None,
        }
    }
}

impl Label {
    pub fn build(self, builder: &mut Builder) {
        let style = builder.style();
        let spacing = builder.spacing();

        let label_rect = style.string_rect(&self.label, style.label_size, None);
        let size = label_rect.size() + spacing.padding.min + spacing.padding.max;

        let layout_rect = builder.layout.push_size(size + spacing.margin.min + spacing.margin.max);
        let bounds = Rect::from_pos_size(layout_rect.pos(), size);

        let at = Rect::from_pos_size(bounds.pos(), label_rect.size()).top_left();

        builder.painter
            .text(self.label, at, &style.font, style.label_size, builder.input().dpi)
            .color(self.color.unwrap_or(style.inactive_foreground))
            .scissor(bounds);
    }
}