use crate::{
    Tab,
    Builder,
};

pub struct Outliner;

impl Outliner {
    pub fn new() -> Self {
        Self
    }
}

impl Tab for Outliner {
    fn name(&self) -> String {
        "Outliner".to_string()
    }

    fn build(&mut self, _builder: &mut Builder) {

    }
}