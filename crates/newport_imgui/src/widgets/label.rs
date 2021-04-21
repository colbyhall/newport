use crate::{ Builder, Padding, Margin, LabelStyle, Alignment };

use crate::math::{ Color, Vector2, Rect };

pub struct Label {
    pub label: String,
    pub color: Option<Color>,
}

impl Label {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();

        Self {
            label: label,
            color: None,
        }
    }

    pub fn build(self, builder: &mut Builder) {
        let padding = builder.style().get::<Padding>().0;
        let margin  = builder.style().get::<Margin>().0;

        let string_rect = LabelStyle::string_rect(builder, &self.label);

        let size = string_rect.size() + padding.min + padding.max + margin.min + margin.max;
        let bounds = builder.layout.push_size(size);
        let bounds: Rect = {
            let min = bounds.min + margin.min;
            let max = bounds.max - margin.max;
            (min, max).into()
        };

        let label_style = builder.style().get::<LabelStyle>().0;

        let at = match label_style.alignment {
            Alignment::Left => {
                bounds.top_left() + Vector2::new(padding.top_left().x, -padding.top_left().y)
            },
            Alignment::Center => {
                let bounds_width = bounds.width() - padding.min.x - padding.max.x;
                let string_width = string_rect.width();

                bounds.top_left() + Vector2::new(padding.top_left().x, -padding.top_left().y) + Vector2::new((bounds_width - string_width) / 2.0, 0.0)
            },
            _ => unimplemented!()
        };

        builder.painter.text(self.label, at, &label_style.font, label_style.size).color(self.color.unwrap_or(label_style.color)).scissor(bounds);
    }
}