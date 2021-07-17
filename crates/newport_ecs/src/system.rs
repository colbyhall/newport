use crate::World;

pub type System = fn(&mut World, f32);

#[derive(Clone)]
pub struct SystemRegister {
    pub(crate) name: &'static str,
    pub(crate) func: System,
    pub(crate) depends_on: Vec<&'static str>,
    pub(crate) active: bool,
}

impl SystemRegister {
    pub fn new(name: &'static str, func: System) -> Self {
        Self {
            name: name,
            func: func,
            depends_on: Vec::new(),
            active: false,
        }
    }

    pub fn depends_on(mut self, name: &'static str) -> Self {
        self.depends_on.push(name);
        self
    }

    pub fn activate(mut self) -> Self {
        self.active = true;
        self
    }
}
