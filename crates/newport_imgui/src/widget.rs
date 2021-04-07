use crate::Id;

pub trait Widget {
    fn frame(&self) -> u64;
    fn id(&self) -> Id;
}