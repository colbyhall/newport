use crate::{
    Builder,
    ColorStyle,
    TextStyle,
    Shape
};

use crate::math::{ Color, Rect };

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
        let color: ColorStyle = builder.style().get();
        let text: TextStyle = builder.style().get();
        
        let label_rect = text.string_rect(&self.label, text.label_size, None).size();
        let bounds = builder.content_bounds(label_rect);

        let at = Rect::from_pos_size(bounds.pos(), label_rect).top_left();

        builder.painter.push_shape(
            Shape::text(
                self.label, 
                at, 
                &text.font, 
                text.label_size, 
                builder.input().dpi, 
                color.inactive_foreground
            )
        );
    }
}