use std::convert::From;

#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct Id(u64);

impl Id {
    pub const NULL: Self = Self(0);
}

impl<T: Sized> From<&T> for Id {
    fn from(t: &T) -> Self {
        Self(t as *const T as u64)
    }
}
