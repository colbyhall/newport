use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hasher, Hash };

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct Id(u64);

impl<T: Hash + ?Sized> From<&T> for Id {
    fn from(t: &T) -> Id {
        let mut hasher = DefaultHasher::new();
        Hash::hash(&t, &mut hasher);
        let id = hasher.finish();
        Id(id)
    }
}