use crate::{
    Tab,
    Builder,
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
        builder.painter.rect(bounds).color(0x000000FF);
    }
}