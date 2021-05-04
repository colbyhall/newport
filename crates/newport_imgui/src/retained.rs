use std::any::Any;

pub trait Retained: RetainedAsAny + 'static {
    fn should_free(&self) -> bool {
        false
    }
}

pub trait RetainedAsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl <T: Retained + 'static> RetainedAsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}