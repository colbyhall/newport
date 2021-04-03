//! Currently aliases to std containers. Eventually copies of std containers using custom allocators
//! 
//! # Todo
//! * Custom container implementations using custom allocators

pub use std::vec::{Vec};
pub use std::collections::{HashMap, HashSet};
pub use std::boxed::Box;

pub use std::alloc::{ Allocator, Layout, AllocError };
use std::ptr::NonNull;
use std::slice::from_raw_parts_mut;
use std::cell::RefCell;

use std::rc::Rc;

struct ArenaInternal {
    memory: Box<[u8]>,
    used:   usize,
}

#[derive(Clone)]
pub struct Arena(Rc<RefCell<ArenaInternal>>);

impl Arena {
    pub fn new(memory: Box<[u8]>) -> Self {
        Self(Rc::new(RefCell::new(ArenaInternal{
            memory: memory,
            used:   0,
        })))
    }

    pub fn reset(&self) {
        assert_eq!(Rc::strong_count(&self.0), 1, "If there is more than one strong reference then memory is in use");
        let mut arena = self.0.as_ref().borrow_mut();
        arena.used = 0;
    }

    pub fn used(&self) -> usize {
        let arena = self.0.as_ref().borrow();
        arena.used
    }
}

unsafe impl Allocator for Arena {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut arena = self.0.as_ref().borrow_mut();

        let used = arena.used;
        let ptr = &mut arena.memory[used] as *mut u8;
        arena.used += layout.size();

        let result: NonNull<[u8]> = unsafe{
            NonNull::new(from_raw_parts_mut(ptr, layout.size()) as *mut [u8]).unwrap()
        };
        
        Ok(result)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // do nothing
    }
}