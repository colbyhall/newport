use newport_editor::{
    Tab,
    Builder,
};

pub struct Details;

impl Details {
    pub fn new() -> Self {
        Self
    }
}

impl Tab for Details {
    fn name(&self) -> String {
        "Details".to_string()
    }

    fn build(&mut self, _builder: &mut Builder) {

    }
}