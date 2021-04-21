use crate::{ Id, Builder, DARK, Padding, Margin, LabelStyle, Alignment };

use crate::math::{ Color, Vector2 };

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
        let padding = builder.style().get::<Padding>().0;
        let margin  = builder.style().get::<Margin>().0;
        let style   = builder.style().get::<ButtonStyle>();

        let string_rect = LabelStyle::string_rect(builder, &self.label);

        let size = string_rect.size() + padding.min + padding.max + margin.min + margin.max;
        let bounds = builder.layout.push_size(size);
        let bounds = {
            let min = bounds.min + margin.min;
            let max = bounds.max - margin.max;
            (min, max).into()
        };

        let mut response = ButtonResponse::None;

        let is_over = builder.input().mouse_is_over(bounds);
        if is_over {
            if !builder.is_hovered(self.id) {
                response = ButtonResponse::Hovered;
            }
            builder.hover(self.id);

            if builder.input().was_primary_clicked() {
                builder.focus(self.id);
            }

            if builder.input().was_primary_released() && builder.unfocus(self.id) {
                response = ButtonResponse::Clicked(0);
            }
        } else {
            builder.unhover(self.id);
        }

        let is_focused = builder.is_focused(self.id);
        
        let (background_color, foreground_color) = {
            let background_color = if is_over {
                if is_focused {
                    style.focused_background
                } else {
                    style.hovered_background
                }
            } else {
                style.unhovered_background
            };

            let foreground_color = if is_over {
                if is_focused {
                    style.focused_foreground
                } else {
                    style.hovered_foreground
                }
            } else {
                style.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.rect(bounds).color(background_color);

        let label_style = builder.style().get::<LabelStyle>().0;

        let at = match label_style.alignment {
            Alignment::Left => {
                bounds.top_left() + Vector2::new(padding.top_left().x, -padding.top_left().y)
            },
            Alignment::Center => {
                let bounds_width = bounds.width() - padding.min.x - padding.max.x;
                let string_width = string_rect.width();

                bounds.top_left() + Vector2::new(0.0, -padding.top_left().y) + Vector2::new((bounds_width - string_width) / 2.0, 0.0)
            },
            _ => unimplemented!()
        };

        builder.painter.text(self.label, at, &label_style.font, label_style.size).color(foreground_color).scissor(bounds);

        response
    }
}

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

#[derive(Clone)]
pub struct ButtonStyle {
    pub unhovered_background: Color,
    pub unhovered_foreground: Color,

    pub hovered_background:   Color,
    pub hovered_foreground:   Color,

    pub focused_background:   Color,
    pub focused_foreground:   Color,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {           
            unhovered_background: DARK.bg,
            hovered_background:   DARK.bg1,
            focused_background:   DARK.bg2,

            unhovered_foreground: DARK.fg,
            hovered_foreground:   DARK.fg2,
            focused_foreground:   DARK.fg3,
        }
    }
}