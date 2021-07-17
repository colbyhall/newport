//! This crate is the HAL for the GPU. Currently Vulkan is the only back end available. The design and architecture
//! was originally concepted after reading http://alextardif.com/RenderingAbstractionLayers.html
//!
//! # Warning
//! 
//! This package is still in a very early state. The API is currently super volatile. I would not 
//! recommend using this package if you don't plan on handling the unknown future changes. 
//! 
//! # Goals
//! 
//! * Abstraction layer should be as lightweight as possible. As many API layer specfic concepts should be 
//!    hidden from the user
//!
//! * Abstraction layer should be as simple as possible. There will be code complexity that is unavoidable but 
//!   they should be rare. If the user ends up spending too much time debugging just to get to the meat of 
//!   their calls then we have failed
//! 
//! * Abstraction layer should be easy to maintain and add on. The hope is that the above points aid this goal
//!
//! # Needs
//! 
//! * Ability to create multiple devices to allow multiple GPU work if desired
//! * Create, upload, and destroy resources (buffers, textures, shaders, pipelines, etc)
//! * Gather, submit, and wait on command work from various passes, in a multicore-compatible way
//! * Automatic device memory management

#![feature(in_band_lifetimes)]

pub(crate) use {
    newport_os as os,
    newport_math as math,
    newport_serde as serde,
    newport_engine as engine,
    newport_asset as asset,
};

use math::{ Rect, Color };

use std::{
    convert::Into,
};

#[cfg(feature = "vulkan")]
mod vk;

#[cfg(feature = "vulkan")]
use vk as api;

mod gpu;
mod shader;
mod context;
mod pipeline;
mod instance;
mod device;
mod receipt;
mod buffer;
mod texture;
mod render_pass;

pub(crate) use {
    shader::*,
};

pub use {
    gpu::*,
    context::*,
    pipeline::*,
    instance::*,
    device::*,
    receipt::*,
    buffer::*,
    texture::*,
    render_pass::*,
};


/// Type of memory allocations that buffers or textures can be allocated from
#[derive(Copy, Clone, Debug)]
pub enum MemoryType {
    /// Able to be uploaded to by mapping memory. Slower to access. Faster to write to
    HostVisible, 
    /// Able to be uploaded to by using commands. Faster to access. Slower to write to
    DeviceLocal,  
}

#[derive(Copy, Clone, Debug)]
pub enum ResourceCreateError {
    Unknown,
    OutOfMemory,
}