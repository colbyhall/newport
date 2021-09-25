use std::collections::hash_map::DefaultHasher;
use std::hash::{
	Hash,
	Hasher,
};

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct Id(u64);

pub trait ToId {
	fn to_id(&self) -> Id;
}

impl<T: Hash + ?Sized> ToId for T {
	fn to_id(&self) -> Id {
		let mut hasher = DefaultHasher::new();
		Hash::hash(self, &mut hasher);
		let id = hasher.finish();
		Id(id)
	}
}

impl<T: Hash + ?Sized> From<&T> for Id {
	fn from(t: &T) -> Id {
		let mut hasher = DefaultHasher::new();
		Hash::hash(&t, &mut hasher);
		let id = hasher.finish();
		Id(id)
	}
}
