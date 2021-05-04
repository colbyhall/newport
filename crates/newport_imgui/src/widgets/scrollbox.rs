use crate::{
    Id,
    ToId,
    Direction,
    Builder,
    Retained,
    Layout,
    ColorStyle,
    Shape,

    math,
};

use math::{
    Rect,
};

pub struct Scrollbox {
    id:        Id,
    bounds:    Rect,
    direction: Direction,
}

impl Scrollbox {
    pub const SIZE: f32 = 20.0;
}

#[derive(Default, Clone)] 
struct ScrollboxRetained {
    current_scroll: f32,
    target_scroll:  f32,
    
    last_used:      f32,
}

impl Retained for ScrollboxRetained { }

impl Scrollbox {
    pub fn new(id: impl ToId, bounds: Rect, direction: Direction)  -> Self {
        Self{
            id: id.to_id(),
            bounds: bounds,
            direction: direction,
        }
    }
}

impl Scrollbox {
    pub fn build(self, builder: &mut Builder, contents: impl FnOnce(&mut Builder)) {
        let color: ColorStyle = builder.style().get();
        builder.painter.push_shape(Shape::solid_rect(builder.layout.bounds(), color.inactive_background, 0.0));

        let retained = builder.retained::<ScrollboxRetained>(self.id);
        let available = match self.direction {
            Direction::LeftToRight|Direction::RightToLeft => {
                self.bounds.width()
            },
            Direction::UpToDown|Direction::DownToUp => {
                self.bounds.height()
            }
        };

        let showing_scrollbar = retained.last_used > available;

        let bounds = if showing_scrollbar {
            self.bounds
        } else {
            self.bounds
        };

        builder.layout(Layout::new(bounds, self.direction), |builder| {
            contents(builder);
        });
    }
}