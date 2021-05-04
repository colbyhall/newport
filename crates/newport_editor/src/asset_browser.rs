use crate::{
    Tab,
    Builder,

    Layout,
    Scrollbox,
    Direction,
};

pub struct AssetBrowser;

impl AssetBrowser {
    pub fn new() -> Self {
        Self
    }
}

impl Tab for AssetBrowser {
    fn name(&self) -> String {
        "Asset Browser".to_string()
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.layout(Layout::left_to_right(builder.layout.bounds()), |builder| {
            let bounds = builder.layout.bounds();

            let size = (300.0, bounds.height()).into();
            Scrollbox::new("test", builder.layout.push_size(size), Direction::UpToDown)
                .build(builder, |builder| {
                    for i in 0..120 {
                        builder.label(format!("Test {}", i));
                    }
                });
        });
    }
}