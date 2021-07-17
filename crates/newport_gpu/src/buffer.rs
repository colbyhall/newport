use crate::{
    api,
};

use std::{
    sync::Arc,
    marker::PhantomData,
};

use bitflags::bitflags;

bitflags! {
    pub struct BufferUsage: u32 {
        const TRANSFER_SRC      = 0b000001;
        const TRANSFER_DST      = 0b000010;
        const VERTEX            = 0b000100;
        const INDEX             = 0b001000;
        const CONSTANTS         = 0b010000;
    }
}

#[derive(Clone)]
pub struct Buffer<T: Sized> {
    pub(crate) api: Arc<api::Buffer>,
    pub(crate) phantom: PhantomData<T>,
    pub(crate) len: usize,
}

impl<T: Sized> Buffer<T> {
    pub fn copy_to(&self, data: &[T]) {
        self.api.copy_to::<T>(data)
    }

    pub fn bindless(&self) -> Option<u32> {
        self.api.bindless()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
