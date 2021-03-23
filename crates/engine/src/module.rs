use std::any::Any;

pub trait Module: Any {
}

pub trait Priority {
    fn foo();
}

