use newport_editor::{
    Tab,
    Builder,

    Shape,
};

pub struct Viewport;

impl Viewport {
    pub fn new() -> Self {
        Self
    }
}

impl Tab for Viewport {
    fn name(&self) -> String {
        "Viewport".to_string()
    }

    fn build(&mut self, builder: &mut Builder) {
        let bounds = builder.layout.bounds();
        builder.painter.push_shape(Shape::solid_rect(bounds, 0x000000FF, 0.0));
    }
}