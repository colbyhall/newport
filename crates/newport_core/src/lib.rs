#![feature(trait_alias)]
//! This crate is the core library all of nevada is built on top of
//! 
//! # Goals
//! 
//! * Custom allocators
//! * Containers using allocators

pub mod containers;
pub mod math;

// use std::any::Any;

// // TODO: Document
// pub trait AsAny: Any + Sized {
//     fn as_any(&self) -> &dyn Any { self }
//     fn as_any_mut(&mut self) -> &mut dyn Any { self }
// }

// impl<T> AsAny for T where T: Any { }