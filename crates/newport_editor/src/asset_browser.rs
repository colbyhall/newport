use crate::{
    Tab,
    Builder,
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

    fn build(&mut self, _builder: &mut Builder) {

    }
}